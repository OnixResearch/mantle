use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use bstr::BStr;
use snix_castore::Node;
use snix_castore::PathComponent;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::fs::fuse::FuseDaemon;
use snix_castore::import::fs::ingest_path;
use snix_castore::refscan::ReferencePattern;
use snix_castore::refscan::ReferenceScanner;
use tokio::io::AsyncReadExt;
use tokio::sync::watch;
use tracing::Span;
use tracing::debug;
use tracing::info;
use tracing::instrument;
use tracing::warn;
use uuid::Uuid;

use super::BuildService;
use super::ephemeral_dir::create_ephemeral_dir;
use crate::buildservice::BuildConstraints;
use crate::buildservice::BuildOutput;
use crate::buildservice::BuildRequest;
use crate::buildservice::BuildResult;
use crate::bwrap::Bwrap;
use crate::bwrap::BwrapCommandFactory;
#[cfg(test)]
use crate::bwrap::watch_cancel::WatchFault;
use crate::sandbox::SandboxMount;
use crate::sandbox::SandboxSpec;
/// Compile-time default for the sandbox shell.
const SANDBOX_SHELL_PLACEHOLDER: &str = "/bin/sh";
const SANDBOX_SHELL_DEFAULT: &str = match option_env!("SNIX_BUILD_SANDBOX_SHELL") {
    Some(path) => path,
    None => SANDBOX_SHELL_PLACEHOLDER,
};
const MAX_INPUT_EXPORT_DEPTH: u32 = 128;
const MAX_INPUT_ROOTS: usize = 65_536;
const MAX_NIX_STORE_SCAN_ENTRIES: u32 = 200_000;

enum ProvidedInputs {
    Fuse { _daemon: FuseDaemon },
    Materialized,
}

/// Resolve the sandbox shell path at runtime.
/// Checks the `SNIX_BUILD_SANDBOX_SHELL` environment variable first,
/// then falls back to the compile-time default. When both are the
/// placeholder [`SANDBOX_SHELL_PLACEHOLDER`], try to discover a static busybox in common
/// NixOS locations so sandboxed builds do not depend on the host glibc.
fn sandbox_shell() -> String {
    let env_shell = std::env::var("SNIX_BUILD_SANDBOX_SHELL").ok();
    let discovered_static = find_static_sandbox_shell();
    choose_sandbox_shell(env_shell.as_deref(), SANDBOX_SHELL_DEFAULT, discovered_static.as_deref())
}

fn choose_sandbox_shell(env_shell: Option<&str>, compile_default: &str, discovered_static: Option<&Path>) -> String {
    if let Some(shell_path) = env_shell
        && shell_path != SANDBOX_SHELL_PLACEHOLDER
    {
        return shell_path.to_string();
    }
    if compile_default != SANDBOX_SHELL_PLACEHOLDER && compile_default_shell_exists(compile_default) {
        return compile_default.to_string();
    }
    if let Some(shell_path) = discovered_static {
        return shell_path.display().to_string();
    }
    SANDBOX_SHELL_PLACEHOLDER.to_string()
}

fn compile_default_shell_exists(compile_default: &str) -> bool {
    if compile_default.is_empty() {
        return false;
    }
    let path = Path::new(compile_default);
    path.is_file()
}

fn find_static_sandbox_shell() -> Option<PathBuf> {
    for candidate in ["/run/current-system/sw/bin/busybox-static", "/bin/busybox.static"] {
        let path = Path::new(candidate);
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }
    find_busybox_static_in_dir(Path::new("/nix/store"))
}

fn find_busybox_static_in_dir(store_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store_dir).ok()?;
    let mut scanned_entries: u32 = 0;
    for entry in entries.flatten() {
        scanned_entries = scanned_entries.saturating_add(1);
        if scanned_entries > MAX_NIX_STORE_SCAN_ENTRIES {
            break;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.contains("busybox-static") {
            continue;
        }
        let candidate = entry.path().join("bin").join("busybox");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn workspace_mounts(
    host_workdir: &Path,
    request: Option<&crate::buildservice::StatefulWorkspaceRequest>,
) -> std::io::Result<Vec<SandboxMount>> {
    let Some(request) = request else {
        return Ok(Vec::new());
    };
    let mount = match request.mode {
        crate::buildservice::StatefulWorkspaceMode::None => return Ok(Vec::new()),
        crate::buildservice::StatefulWorkspaceMode::ImmutableSnapshot => {
            let input_name = request
                .snapshot_input_name
                .as_ref()
                .ok_or_else(|| std::io::Error::other("immutable workspace snapshot input is missing"))?;
            if input_name.is_absolute()
                || input_name.components().any(|component| {
                    matches!(component, std::path::Component::CurDir | std::path::Component::ParentDir)
                })
            {
                return Err(std::io::Error::other("immutable workspace snapshot input must be a clean relative path"));
            }
            SandboxMount {
                host_path: host_workdir.join("host_inputs_dir").join(input_name),
                guest_path: request.guest_path.clone(),
                read_only: true,
            }
        }
        crate::buildservice::StatefulWorkspaceMode::MutableSession => SandboxMount {
            host_path: request
                .runtime_host_path
                .clone()
                .ok_or_else(|| std::io::Error::other("mutable workspace host path was not resolved by the shell"))?,
            guest_path: request.guest_path.clone(),
            read_only: false,
        },
    };
    Ok(vec![mount])
}

fn compile_socket_mount(host_socket: &Path) -> std::io::Result<Option<SandboxMount>> {
    use std::os::unix::fs::FileTypeExt as _;
    match std::fs::symlink_metadata(host_socket) {
        Ok(metadata) if metadata.file_type().is_socket() => Ok(Some(SandboxMount {
            host_path: host_socket.to_path_buf(),
            guest_path: PathBuf::from("/run/mantle/compile-cache.sock"),
            read_only: true,
        })),
        Ok(_) => Err(std::io::Error::other(
            "compile runtime endpoint must be a Unix socket, not a file, directory, or symlink",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn rust_wait_socket_mount(host_socket: &Path) -> std::io::Result<SandboxMount> {
    use std::os::unix::fs::FileTypeExt as _;
    if !std::fs::symlink_metadata(host_socket)?.file_type().is_socket() {
        return Err(std::io::Error::other(
            "Rust wait endpoint must be a Unix socket, not a file, directory, or symlink",
        ));
    }
    Ok(SandboxMount {
        host_path: host_socket.to_path_buf(),
        guest_path: PathBuf::from("/run/mantle/rust-waits.sock"),
        read_only: true,
    })
}

fn should_materialize_inputs_fallback(error: &std::io::Error) -> bool {
    if error.kind() != ErrorKind::Other {
        return false;
    }
    let message = error.to_string();
    if !message.contains("fusermount") {
        return false;
    }
    message.contains("Unexpected exit code when running fusermount")
}

async fn materialize_inputs_to_disk<BS, DS>(
    inputs_root: &Path,
    root_nodes: &BTreeMap<PathComponent, Node>,
    blob_service: &BS,
    directory_service: &DS,
) -> std::io::Result<()>
where
    BS: BlobService + Clone,
    DS: DirectoryService + Clone,
{
    assert!(root_nodes.len() <= MAX_INPUT_ROOTS, "input root count exceeded {}", MAX_INPUT_ROOTS,);
    std::fs::create_dir_all(inputs_root)?;
    for (name, node) in root_nodes {
        let name_str = std::str::from_utf8(name.as_ref()).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid input path component: {e}"))
        })?;
        let dest = inputs_root.join(name_str);
        materialize_node_to_disk(&dest, node, blob_service, directory_service, 0).await?;
    }
    Ok(())
}

async fn materialize_node_to_disk<BS, DS>(
    dest: &Path,
    node: &Node,
    blob_service: &BS,
    directory_service: &DS,
    depth: u32,
) -> std::io::Result<()>
where
    BS: BlobService + Clone,
    DS: DirectoryService + Clone,
{
    if depth >= MAX_INPUT_EXPORT_DEPTH {
        return Err(std::io::Error::other(format!(
            "input export depth limit ({MAX_INPUT_EXPORT_DEPTH}) exceeded at {}",
            dest.display(),
        )));
    }
    match node {
        Node::File { digest, executable, .. } => {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut reader = blob_service
                .open_read(digest)
                .await?
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("blob {digest} not found")))?;
            let mut file = std::fs::File::create(dest)?;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let read_bytes = reader.read(&mut buf).await?;
                if read_bytes == 0 {
                    break;
                }
                std::io::Write::write_all(&mut file, &buf[..read_bytes])?;
            }
            #[cfg(unix)]
            if *executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o555))?;
            }
        }
        Node::Directory { digest, .. } => {
            std::fs::create_dir_all(dest)?;
            let directory = directory_service.get(digest).await.map_err(std::io::Error::other)?.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, format!("directory {digest} not found"))
            })?;
            for (name, child_node) in directory.nodes() {
                let name_str = std::str::from_utf8(name.as_ref()).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid directory entry name: {e}"))
                })?;
                let child_dest = dest.join(name_str);
                Box::pin(materialize_node_to_disk(
                    &child_dest,
                    child_node,
                    blob_service,
                    directory_service,
                    depth.saturating_add(1),
                ))
                .await?;
            }
        }
        Node::Symlink { target, .. } => {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let target_os = std::ffi::OsStr::from_bytes(target.as_ref());
                std::os::unix::fs::symlink(target_os, dest)?;
            }
        }
    }
    Ok(())
}

const MATERIALIZATION_WORKER_NAME: &str = "snix-build-input-materializer";
const MATERIALIZATION_WORKER_PANIC: &str = "input materialization worker panicked";

fn materialize_inputs_blocking<BS, DS>(
    inputs_root: &Path,
    root_nodes: &BTreeMap<PathComponent, Node>,
    blob_service: &BS,
    directory_service: &DS,
) -> std::io::Result<()>
where
    BS: BlobService + Clone + Send + 'static,
    DS: DirectoryService + Clone + Send + 'static,
{
    let inputs_root = inputs_root.to_path_buf();
    let root_nodes = root_nodes.clone();
    let blob_service = blob_service.clone();
    let directory_service = directory_service.clone();
    let worker = std::thread::Builder::new().name(MATERIALIZATION_WORKER_NAME.to_string()).spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        runtime.block_on(materialize_inputs_to_disk(&inputs_root, &root_nodes, &blob_service, &directory_service))
    })?;
    worker.join().map_err(|_| std::io::Error::other(MATERIALIZATION_WORKER_PANIC))?
}

/// Unified workspace guard shared with the sandbox supervisor until PID1,
/// start-gate, and cgroup teardown complete.
type SupervisedWorkspace = std::sync::Arc<crate::bwrap::watch_cancel::WatchWorkspace>;

/// Default local sandbox concurrency when callers do not supply a job policy.
pub const DEFAULT_CONCURRENT_SANDBOX_BUILDS: u32 = 2;
/// Conservative per-sandbox file-descriptor allowance (FUSE, pipes, bwrap, proc).
pub const ESTIMATED_FDS_PER_SANDBOX: u32 = 32;
/// Default file-descriptor budget reserved for concurrent local sandboxes.
pub const DEFAULT_SANDBOX_FD_BUDGET: u32 = 1024;
/// Hard upper bound on concurrent local sandboxes under the default FD budget.
pub const MAX_CONCURRENT_SANDBOX_BUILDS: u32 = DEFAULT_SANDBOX_FD_BUDGET / ESTIMATED_FDS_PER_SANDBOX;

/// Explicit local sandbox admission policy combining requested jobs with a
/// declared file-descriptor safety budget so excess work queues rather than
/// exhausting open file descriptors (`EMFILE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxConcurrencyPolicy {
    pub requested_jobs: u32,
    pub fd_budget: u32,
}

impl SandboxConcurrencyPolicy {
    pub const fn for_jobs(requested_jobs: u32) -> Self {
        Self {
            requested_jobs,
            fd_budget: DEFAULT_SANDBOX_FD_BUDGET,
        }
    }

    pub fn effective_permits(&self) -> std::io::Result<u32> {
        if self.requested_jobs == 0 {
            return Err(std::io::Error::new(ErrorKind::InvalidInput, "sandbox job limit must be at least 1"));
        }
        if self.fd_budget < ESTIMATED_FDS_PER_SANDBOX {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "sandbox file-descriptor budget {} is below minimum {} per sandbox",
                    self.fd_budget, ESTIMATED_FDS_PER_SANDBOX
                ),
            ));
        }
        let fd_cap = self.fd_budget / ESTIMATED_FDS_PER_SANDBOX;
        Ok(self.requested_jobs.min(fd_cap))
    }
}

pub struct BubblewrapBuildService<BS, DS> {
    /// Root path in which all builds run
    workdir: PathBuf,

    /// Handle to a [BlobService], used by filesystems spawned during builds.
    blob_service: BS,
    /// Handle to a [DirectoryService], used by filesystems spawned during builds.
    directory_service: DS,

    /// Optional worker-owned root where failed sandboxes are retained before
    /// an upper-layer diagnostic shell ingests allowlisted evidence.
    failure_workspace_root: Option<PathBuf>,

    /// Explicit execution-only endpoint; never serialized in BuildRequest.
    compile_socket: Option<PathBuf>,
    /// Optional diagnostic transport; never part of derivation or cache identity.
    rust_wait_socket: Option<PathBuf>,
    /// Execution-only launcher policy, applied after input population.
    command_factory: Option<Arc<dyn BwrapCommandFactory>>,

    /// Effective permit count backing `concurrent_builds`.
    max_concurrent_builds: u32,
    // semaphore to track number of concurrently running builds under job + FD policy.
    concurrent_builds: std::sync::Arc<tokio::sync::Semaphore>,
    /// Local execution deadlines and staged capability-aware resource policy.
    execution_policy: super::SandboxExecutionPolicy,
    /// Optional external supervisor graceful-shutdown trigger.
    shutdown_signal: Option<watch::Receiver<bool>>,
    /// Most recent attempt's cgroup/resource enforcement facts.
    last_enforcement_facts: std::sync::Arc<std::sync::Mutex<Option<super::SandboxResourceEnforcementFacts>>>,
    /// Optional bounded opt-in live CPU diagnostics sink (never affects BuildRequest identity).
    live_cpu_diagnostics:
        Option<(super::SandboxLiveCpuDiagnosticsConfig, tokio::sync::mpsc::Sender<super::SandboxCpuSampleEvent>)>,
    /// Most recent attempt's live CPU diagnostics facts.
    last_live_cpu_facts: std::sync::Arc<std::sync::Mutex<Option<super::SandboxLiveCpuAttemptFacts>>>,
    #[cfg(test)]
    watch_fault: Option<WatchFault>,
}

static NEXT_LIVE_CPU_ATTEMPT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl<BS, DS> BubblewrapBuildService<BS, DS> {
    pub fn new(workdir: PathBuf, blob_service: BS, directory_service: DS) -> Self {
        // We map root inside the container to the uid/gid this is running at,
        // and allocate one for uid 1000 into the container from the range we
        // got in /etc/sub{u,g}id.
        // FUTUREWORK: use different uids?
        Self {
            workdir,
            blob_service,
            directory_service,
            failure_workspace_root: None,
            compile_socket: None,
            rust_wait_socket: None,
            command_factory: None,
            max_concurrent_builds: DEFAULT_CONCURRENT_SANDBOX_BUILDS,
            concurrent_builds: std::sync::Arc::new(tokio::sync::Semaphore::new(
                DEFAULT_CONCURRENT_SANDBOX_BUILDS as usize,
            )),
            execution_policy: super::SandboxExecutionPolicy::standard(),
            shutdown_signal: None,
            last_enforcement_facts: std::sync::Arc::new(std::sync::Mutex::new(None)),
            live_cpu_diagnostics: None,
            last_live_cpu_facts: std::sync::Arc::new(std::sync::Mutex::new(None)),
            #[cfg(test)]
            watch_fault: None,
        }
    }

    /// Enable opt-in bounded live CPU diagnostics for owned local attempts.
    pub fn with_live_cpu_diagnostics(
        mut self,
        config: super::SandboxLiveCpuDiagnosticsConfig,
        sender: tokio::sync::mpsc::Sender<super::SandboxCpuSampleEvent>,
    ) -> std::io::Result<Self> {
        config.validate()?;
        self.live_cpu_diagnostics = Some((config, sender));
        Ok(self)
    }

    /// Return the live CPU sampler facts recorded for the latest local attempt.
    pub fn last_live_cpu_facts(&self) -> Option<super::SandboxLiveCpuAttemptFacts> {
        self.last_live_cpu_facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    /// Configure local execution deadlines and staged resource enforcement policy.
    pub fn with_execution_policy(mut self, policy: super::SandboxExecutionPolicy) -> std::io::Result<Self> {
        policy.validate()?;
        self.execution_policy = policy;
        Ok(self)
    }

    /// Return the configured local execution policy.
    pub fn execution_policy(&self) -> &super::SandboxExecutionPolicy {
        &self.execution_policy
    }

    /// Attach an external supervisor graceful-shutdown signal.
    pub fn with_shutdown_signal(mut self, shutdown_signal: watch::Receiver<bool>) -> Self {
        self.shutdown_signal = Some(shutdown_signal);
        self
    }

    /// Return the resource enforcement facts recorded for the latest local attempt.
    pub fn last_enforcement_facts(&self) -> Option<super::SandboxResourceEnforcementFacts> {
        self.last_enforcement_facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    /// Size local sandbox concurrency from the resolved job policy under the
    /// default file-descriptor safety budget.
    pub fn with_max_concurrent_builds(self, requested_jobs: u32) -> std::io::Result<Self> {
        self.with_concurrency_policy(SandboxConcurrencyPolicy::for_jobs(requested_jobs))
    }

    /// Size local sandbox concurrency from an explicit job + FD budget policy.
    pub fn with_concurrency_policy(mut self, policy: SandboxConcurrencyPolicy) -> std::io::Result<Self> {
        let permits = policy.effective_permits()?;
        self.max_concurrent_builds = permits;
        self.concurrent_builds = std::sync::Arc::new(tokio::sync::Semaphore::new(permits as usize));
        Ok(self)
    }

    /// Return the effective concurrent sandbox admission limit.
    pub fn max_concurrent_builds(&self) -> u32 {
        self.max_concurrent_builds
    }

    /// Opt in to one socket, never a cache/state directory or arbitrary guest path.
    /// A missing endpoint is deliberately left unmounted so clients can degrade.
    pub fn with_compile_socket(mut self, host_socket: PathBuf) -> std::io::Result<Self> {
        if !host_socket.is_absolute()
            || host_socket.as_os_str().as_encoded_bytes().len() > 107
            || host_socket.as_os_str().as_encoded_bytes().contains(&0)
            || host_socket
                .components()
                .any(|c| matches!(c, std::path::Component::CurDir | std::path::Component::ParentDir))
        {
            return Err(std::io::Error::other("compile socket path must be a bounded clean absolute path"));
        }
        compile_socket_mount(&host_socket)?;
        self.compile_socket = Some(host_socket);
        Ok(self)
    }

    /// Opt in to one execution-only diagnostic endpoint, independent of compile caching.
    /// The endpoint must remain an existing non-symlink Unix socket until launch.
    pub fn with_rust_wait_socket(mut self, host_socket: PathBuf) -> std::io::Result<Self> {
        if !host_socket.is_absolute()
            || host_socket.as_os_str().as_encoded_bytes().len() > 107
            || host_socket.as_os_str().as_encoded_bytes().contains(&0)
            || host_socket
                .components()
                .any(|c| matches!(c, std::path::Component::CurDir | std::path::Component::ParentDir))
        {
            return Err(std::io::Error::other("Rust wait socket path must be a bounded clean absolute path"));
        }
        rust_wait_socket_mount(&host_socket)?;
        self.rust_wait_socket = Some(host_socket);
        Ok(self)
    }

    /// Opt in to a command factory without changing serialized build requests.
    /// The callback runs only after this sandbox's inputs are available.
    pub fn with_command_factory(mut self, command_factory: Arc<dyn BwrapCommandFactory>) -> Self {
        self.command_factory = Some(command_factory);
        self
    }

    pub fn with_failure_workspace_root(mut self, failure_workspace_root: PathBuf) -> Self {
        self.failure_workspace_root = Some(failure_workspace_root);
        self
    }
}

impl<BS, DS> BubblewrapBuildService<BS, DS>
where
    BS: BlobService + Clone + 'static,
    DS: DirectoryService + Clone + 'static,
{
    #[instrument(skip_all, err)]
    async fn do_build_inner(
        &self,
        request: BuildRequest,
        cancellation: Option<watch::Receiver<bool>>,
    ) -> std::io::Result<BuildResult> {
        if is_watch_triggered(cancellation.as_ref()) || is_watch_triggered(self.shutdown_signal.as_ref()) {
            return Err(watch_cancelled());
        }
        let admission_started = std::time::Instant::now();
        let permit = std::sync::Arc::clone(&self.concurrent_builds)
            .acquire_owned()
            .await
            .map_err(|error| std::io::Error::other(format!("sandbox concurrency semaphore closed: {error}")))?;
        let admission_wait_ms = u64::try_from(admission_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            admission_wait_ms,
            max_concurrent_builds = self.max_concurrent_builds,
            "local sandbox admission acquired"
        );
        let execution_started = std::time::Instant::now();
        if is_watch_triggered(cancellation.as_ref()) || is_watch_triggered(self.shutdown_signal.as_ref()) {
            drop(permit);
            return Err(watch_cancelled());
        }

        let build_name = Uuid::new_v4();
        let build_name_str = build_name.to_string();
        let debug_failure_workspace_root = std::env::var_os("CRUNCH_DEBUG_FAILURE_WORKSPACE_ROOT").map(PathBuf::from);
        let failure_workspace_root = self.failure_workspace_root.as_deref().or(debug_failure_workspace_root.as_deref());
        let sandbox_dir = create_ephemeral_dir(&self.workdir, &format!("{build_name_str}-"))
            .map_err(|error| super::wrap_classified_io_error(error, super::SandboxFailureClass::SandboxStartup))?;
        let sandbox_dir: SupervisedWorkspace =
            std::sync::Arc::new(crate::bwrap::watch_cancel::WatchWorkspace::new(sandbox_dir));
        info!(build_name = %build_name_str, sandbox_path = %sandbox_dir.path().display(), "Starting bwrap build");

        let span = Span::current();
        span.record("build_name", build_name_str.clone());

        let blob_service = self.blob_service.clone();
        let directory_service = self.directory_service.clone();
        let mut workspace_mounts = workspace_mounts(sandbox_dir.path(), request.workspace.as_ref())
            .map_err(|error| super::wrap_classified_io_error(error, super::SandboxFailureClass::SandboxStartup))?;
        if self.rust_wait_socket.is_some()
            && workspace_mounts.iter().any(|mount| {
                mount.guest_path.starts_with("/run/mantle") || Path::new("/run/mantle").starts_with(&mount.guest_path)
            })
        {
            return Err(super::classified_io_error(
                std::io::ErrorKind::Other,
                super::SandboxFailureClass::SandboxStartup,
                "workspace overlaps Rust wait runtime socket",
            ));
        }
        if let Some(host_socket) = &self.compile_socket {
            if workspace_mounts.iter().any(|mount| {
                mount.guest_path.starts_with("/run/mantle") || Path::new("/run/mantle").starts_with(&mount.guest_path)
            }) {
                return Err(super::classified_io_error(
                    std::io::ErrorKind::Other,
                    super::SandboxFailureClass::SandboxStartup,
                    "workspace overlaps compile runtime socket",
                ));
            }
            if let Some(mount) = compile_socket_mount(host_socket)
                .map_err(|error| super::wrap_classified_io_error(error, super::SandboxFailureClass::SandboxStartup))?
            {
                workspace_mounts.push(mount);
            }
        }
        if let Some(host_socket) = &self.rust_wait_socket {
            workspace_mounts.push(
                rust_wait_socket_mount(host_socket).map_err(|error| {
                    super::wrap_classified_io_error(error, super::SandboxFailureClass::SandboxStartup)
                })?,
            );
        }

        let spec = SandboxSpec::builder()
            .host_workdir(sandbox_dir.path().to_path_buf())
            .sandbox_workdir(request.working_dir)
            .scratches(request.scratch_paths)
            .command(request.command_args)
            .env_vars(request.environment_vars)
            .additional_files(request.additional_files)
            .mounts(workspace_mounts)
            .with_inputs(request.inputs_dir, move |path| {
                let root_nodes = request.inputs.clone();
                let fs = snix_castore::fs::SnixStoreFs::new(
                    blob_service.clone(),
                    directory_service.clone(),
                    Box::new(root_nodes.clone()),
                    snix_castore::fs::FSSettings {
                        list_root: true,
                        uid_gid_override: None,
                        show_xattr: false,
                    },
                    tokio::runtime::Handle::current(),
                );
                // FUTUREWORK: make fuse daemon threads configurable?
                if std::env::var_os("CRUNCH_NO_FUSE").is_some() {
                    info!(?path, "CRUNCH_NO_FUSE set, materializing inputs to disk");
                    materialize_inputs_blocking(path, &root_nodes, &blob_service, &directory_service)?;
                    Ok(ProvidedInputs::Materialized)
                } else {
                    match FuseDaemon::new(fs, path, 4, false) {
                        Ok(daemon) => Ok(ProvidedInputs::Fuse { _daemon: daemon }),
                        Err(error) => {
                            if !should_materialize_inputs_fallback(&error) {
                                return Err(error);
                            }
                            warn!(?error, ?path, "fusermount mount failed, materializing inputs to disk");
                            materialize_inputs_blocking(path, &root_nodes, &blob_service, &directory_service)?;
                            Ok(ProvidedInputs::Materialized)
                        }
                    }
                }
            })
            .allow_network(request.constraints.contains(&BuildConstraints::NetworkAccess))
            .dns_config(request.dns_config.clone())
            .provide_proc_metadata(request.constraints.contains(&BuildConstraints::ProvideProcMetadata))
            .provide_random_devices(request.constraints.contains(&BuildConstraints::ProvideRandomDevices))
            .provide_shell(
                request.constraints.contains(&BuildConstraints::ProvideBinSh).then_some(sandbox_shell().into()),
            )
            .build();

        let sandbox = Bwrap::initialize(spec)
            .map_err(|error| super::wrap_classified_io_error(error, super::SandboxFailureClass::SandboxStartup))?;
        let sandbox = match &self.command_factory {
            Some(factory) => sandbox.with_command_factory(Arc::clone(factory)),
            None => sandbox,
        };
        let live_cpu = self.live_cpu_diagnostics.as_ref().map(|(config, sender)| {
            let attempt_id = NEXT_LIVE_CPU_ATTEMPT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut effective_config = config.clone();
            let attempt_label = match super::current_live_cpu_attempt_context() {
                Some(ctx) => {
                    effective_config.generation = ctx.generation;
                    ctx.attempt_label
                }
                None => build_name_str.clone(),
            };
            crate::bwrap::BwrapLiveCpuDiagnostics {
                config: effective_config,
                attempt_id,
                attempt_label,
                sender: sender.clone(),
                facts_sink: Some(Arc::clone(&self.last_live_cpu_facts)),
            }
        });
        let sandbox = sandbox
            .with_execution_policy(self.execution_policy.clone())
            .with_shutdown_signal(self.shutdown_signal.clone())
            .with_helper_sockets(self.compile_socket.clone(), self.rust_wait_socket.clone())
            .with_enforcement_facts_sink(Some(Arc::clone(&self.last_enforcement_facts)))
            .with_live_cpu_diagnostics(live_cpu);
        #[cfg(test)]
        let sandbox = sandbox.with_test_watch_fault(self.watch_fault.filter(|_| cancellation.is_some()));
        let lease = crate::bwrap::watch_cancel::WatchLease::new(permit, Arc::clone(&sandbox_dir));
        let outcome = sandbox
            .run_supervised(cancellation.clone(), lease)
            .await
            .map_err(|error| super::wrap_classified_io_error(error, super::SandboxFailureClass::SandboxStartup))?;
        if is_watch_triggered(cancellation.as_ref()) || is_watch_triggered(self.shutdown_signal.as_ref()) {
            return Err(watch_cancelled());
        }

        if !outcome.output().status.success() {
            let stdout = BStr::new(&outcome.output().stdout);
            let stderr = BStr::new(&outcome.output().stderr);

            warn!(stdout=%stdout, stderr=%stderr, exit_code=%outcome.output().status, "build failed");

            // Include stdout+stderr in the error so callers can display the
            // build log. Previously this was just "nonzero exit code".
            let mut log = String::new();
            if !outcome.output().stdout.is_empty() {
                log.push_str(&String::from_utf8_lossy(&outcome.output().stdout));
            }
            if !outcome.output().stderr.is_empty() {
                if !log.is_empty() {
                    log.push('\n');
                }
                log.push_str(&String::from_utf8_lossy(&outcome.output().stderr));
            }
            let msg = if log.is_empty() {
                format!("nonzero exit code: {}", outcome.output().status)
            } else {
                format!("nonzero exit code: {}\n{}", outcome.output().status, log)
            };
            let tempdir = std::sync::Arc::try_unwrap(sandbox_dir)
                .expect("supervisor finished before retaining failed workspace")
                .into_tempdir();
            retain_failed_sandbox(tempdir, failure_workspace_root, &build_name_str);
            return Err(super::classified_io_error(
                std::io::ErrorKind::Other,
                super::SandboxFailureClass::Builder,
                msg,
            ));
        }

        let outputs: Vec<_> = request.outputs.iter().filter_map(|o| outcome.find_path(o)).collect();
        if outputs.len() != request.outputs.len() {
            warn!("Not all outputs produced");
            let tempdir = std::sync::Arc::try_unwrap(sandbox_dir)
                .expect("supervisor finished before retaining failed workspace")
                .into_tempdir();
            retain_failed_sandbox(tempdir, failure_workspace_root, &build_name_str);
            return Err(super::classified_io_error(
                std::io::ErrorKind::Other,
                super::SandboxFailureClass::MissingOutputs,
                "Not all outputs produced",
            ));
        }
        let patterns = ReferencePattern::new(request.refscan_needles);
        let outputs = futures::future::try_join_all(outputs.into_iter().enumerate().map(|(i, host_output_path)| {
            let output_path = &request.outputs[i];
            debug!(host.path=?host_output_path, output.path=?output_path, "ingesting path");
            let patterns = patterns.clone();
            async move {
                let scanner = ReferenceScanner::new(patterns);
                Ok::<_, std::io::Error>(BuildOutput {
                    node: ingest_path(&self.blob_service, &self.directory_service, host_output_path, Some(&scanner))
                        .await
                        .map_err(|e| {
                            super::classified_io_error(
                                std::io::ErrorKind::InvalidData,
                                super::SandboxFailureClass::SandboxStartup,
                                format!("Unable to ingest output: {e}"),
                            )
                        })?,

                    output_needles: scanner
                        .matches()
                        .into_iter()
                        .enumerate()
                        .filter(|(_, val)| *val)
                        .map(|(idx, _)| idx as u64)
                        .collect(),
                })
            }
        }))
        .await?;
        if is_watch_triggered(cancellation.as_ref()) || is_watch_triggered(self.shutdown_signal.as_ref()) {
            return Err(watch_cancelled());
        }
        // Capture stdout+stderr for the log (even on success)
        let mut log = String::new();
        if !outcome.output().stdout.is_empty() {
            log.push_str(&String::from_utf8_lossy(&outcome.output().stdout));
        }
        if !outcome.output().stderr.is_empty() {
            if !log.is_empty() {
                log.push('\n');
            }
            log.push_str(&String::from_utf8_lossy(&outcome.output().stderr));
        }

        if is_watch_triggered(cancellation.as_ref()) || is_watch_triggered(self.shutdown_signal.as_ref()) {
            return Err(watch_cancelled());
        }
        let execution_ms = u64::try_from(execution_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            admission_wait_ms,
            execution_ms,
            max_concurrent_builds = self.max_concurrent_builds,
            "local sandbox execution completed"
        );
        Ok(BuildResult {
            outputs,
            log: if log.is_empty() { None } else { Some(log) },
            timing: Some(super::SandboxExecutionTiming {
                admission_wait_ms,
                execution_ms,
            }),
        })
    }
}

fn is_watch_triggered(receiver: Option<&watch::Receiver<bool>>) -> bool {
    receiver.is_some_and(|r| *r.borrow() || r.has_changed().is_err())
}

fn watch_cancelled() -> std::io::Error {
    std::io::Error::new(ErrorKind::Interrupted, "watch build cancelled before output admission")
}

#[async_trait]
impl<BS, DS> BuildService for BubblewrapBuildService<BS, DS>
where
    BS: BlobService + Clone + 'static,
    DS: DirectoryService + Clone + 'static,
{
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.do_build_inner(request, None).await
    }

    async fn do_build_cancellable(
        &self,
        request: BuildRequest,
        cancellation: watch::Receiver<bool>,
    ) -> std::io::Result<BuildResult> {
        self.do_build_inner(request, Some(cancellation)).await
    }
}

fn retain_failed_sandbox(sandbox_dir: tempfile::TempDir, root: Option<&Path>, build_name: &str) {
    let Some(root) = root else {
        return;
    };
    let sandbox_path = sandbox_dir.keep();
    if let Err(error) = std::fs::create_dir_all(root) {
        warn!(?error, "failed to create diagnostic failure-workspace root");
        let _ = std::fs::remove_dir_all(&sandbox_path);
        return;
    }
    let destination = root.join(build_name);
    if let Err(error) = std::fs::rename(&sandbox_path, &destination) {
        warn!(?error, "failed to retain diagnostic failure workspace");
        let _ = std::fs::remove_dir_all(&sandbox_path);
        return;
    }
    info!(failure_workspace = %destination.display(), "retained failed sandbox for bounded diagnostic capture");
}

#[cfg(test)]
mod tests {

    #[test]
    fn compile_endpoint_admits_only_socket_and_degrades_after_removal() {
        use std::os::unix::net::UnixListener;
        let root = tempfile::tempdir().unwrap();
        let endpoint = root.path().join("daemon.sock");
        let listener = UnixListener::bind(&endpoint).unwrap();
        let mount = compile_socket_mount(&endpoint).unwrap().unwrap();
        assert_eq!(mount.host_path, endpoint);
        assert_eq!(mount.guest_path, PathBuf::from("/run/mantle/compile-cache.sock"));
        assert!(mount.read_only);
        drop(listener);
        std::fs::remove_file(&endpoint).unwrap();
        assert!(compile_socket_mount(&endpoint).unwrap().is_none());
        std::fs::write(&endpoint, b"not a socket").unwrap();
        assert!(compile_socket_mount(&endpoint).err().unwrap().to_string().contains("Unix socket"));
        assert!(compile_socket_mount(root.path()).err().unwrap().to_string().contains("Unix socket"));
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&endpoint, &alias).unwrap();
        assert!(compile_socket_mount(&alias).err().unwrap().to_string().contains("Unix socket"));
    }

    #[test]
    fn rust_wait_endpoint_rejects_invalid_paths_and_replaced_sockets() {
        use std::os::unix::ffi::OsStringExt as _;
        use std::os::unix::net::UnixListener;
        let root = tempfile::tempdir().unwrap();
        let service = || BubblewrapBuildService::new(root.path().to_path_buf(), (), ());
        for path in [
            PathBuf::from("relative.sock"),
            root.path().join("../wait.sock"),
            PathBuf::from(format!("/{}", "a".repeat(107))),
            PathBuf::from(std::ffi::OsString::from_vec(b"/wait\0.sock".to_vec())),
            root.path().join("missing.sock"),
            root.path().to_path_buf(),
        ] {
            assert!(service().with_rust_wait_socket(path).is_err());
        }
        let endpoint = root.path().join("wait.sock");
        let listener = UnixListener::bind(&endpoint).unwrap();
        let _service = service().with_rust_wait_socket(endpoint.clone()).unwrap();
        let alias = root.path().join("alias.sock");
        std::os::unix::fs::symlink(&endpoint, &alias).unwrap();
        assert!(service().with_rust_wait_socket(alias).is_err());
        drop(listener);
        std::fs::remove_file(&endpoint).unwrap();
        assert_eq!(rust_wait_socket_mount(&endpoint).unwrap_err().kind(), ErrorKind::NotFound);
        std::fs::write(&endpoint, b"not a socket").unwrap();
        assert!(rust_wait_socket_mount(&endpoint).is_err());
        std::fs::remove_file(&endpoint).unwrap();
        std::os::unix::fs::symlink(root.path(), &endpoint).unwrap();
        assert!(rust_wait_socket_mount(&endpoint).is_err());
    }

    #[tokio::test]
    async fn real_buildservice_mounts_rust_wait_and_compile_endpoints_independently() {
        use std::os::unix::net::UnixListener;
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let compile = root.path().join("compile.sock");
        let waits = root.path().join("wait.sock");
        let _compile_listener = UnixListener::bind(&compile).unwrap();
        let _wait_listener = UnixListener::bind(&waits).unwrap();
        for (compile_enabled, waits_enabled) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut service =
                BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
            if compile_enabled {
                service = service.with_compile_socket(compile.clone()).unwrap();
            }
            if waits_enabled {
                service = service.with_rust_wait_socket(waits.clone()).unwrap();
            }
            let request = BuildRequest {
                working_dir: PathBuf::from("build"),
                scratch_paths: vec![PathBuf::from("build"), PathBuf::from("nix/store")],
                inputs_dir: PathBuf::from("nix/store"),
                outputs: vec![PathBuf::from("nix/store/runtime-result")],
                constraints: std::iter::once(BuildConstraints::ProvideBinSh).collect(),
                command_args: vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    format!(
                        "test {}-S /run/mantle/compile-cache.sock && test {}-S /run/mantle/rust-waits.sock && echo runtime-sockets-visible && echo done > /nix/store/runtime-result",
                        if compile_enabled { "" } else { "! " },
                        if waits_enabled { "" } else { "! " },
                    ),
                ],
                ..Default::default()
            };
            let result = service.do_build(request).await.unwrap();
            assert_eq!(result.log.as_deref(), Some("runtime-sockets-visible\n"));
        }
    }

    use snix_castore::Directory;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use tokio::io::AsyncWriteExt;

    use super::*;

    const TEST_DIGEST_HEX_LENGTH: usize = 64;

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("snix-build-bwrap-tests".to_string(), RedbDirectoryServiceConfig::default())
            .unwrap()
    }

    #[test]
    fn failed_sandbox_retention_moves_workspace_only_when_configured() {
        let root = tempfile::tempdir().unwrap();
        let sandbox = tempfile::tempdir().unwrap();
        std::fs::write(sandbox.path().join("trace.json"), b"bounded diagnostic").unwrap();
        retain_failed_sandbox(sandbox, Some(root.path()), "attempt-a");

        assert_eq!(std::fs::read(root.path().join("attempt-a/trace.json")).unwrap(), b"bounded diagnostic");
        assert!(root.path().join("attempt-a").is_dir());
    }

    #[test]
    fn failed_sandbox_without_retention_root_is_cleaned_by_tempdir_drop() {
        let sandbox = tempfile::tempdir().unwrap();
        let path = sandbox.path().to_path_buf();
        retain_failed_sandbox(sandbox, None, "attempt-b");

        assert!(!path.exists());
        assert!(!path.is_dir());
    }

    async fn insert_blob(bs: &MemoryBlobService, data: &[u8], executable: bool) -> Node {
        let mut writer = bs.open_write().await;
        writer.write_all(data).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: data.len() as u64,
            executable,
        }
    }

    #[test]
    fn choose_sandbox_shell_prefers_explicit_env() {
        let chosen = choose_sandbox_shell(
            Some("/nix/store/abc-static-bash/bin/bash"),
            "/bin/sh",
            Some(Path::new("/nix/store/def-busybox-static/bin/busybox")),
        );
        assert_eq!(chosen, "/nix/store/abc-static-bash/bin/bash");
    }

    #[test]
    fn choose_sandbox_shell_uses_discovered_static_for_placeholder() {
        let chosen = choose_sandbox_shell(
            Some("/bin/sh"),
            "/bin/sh",
            Some(Path::new("/nix/store/def-busybox-static/bin/busybox")),
        );
        assert_eq!(chosen, "/nix/store/def-busybox-static/bin/busybox");
    }

    #[test]
    fn choose_sandbox_shell_ignores_missing_compile_default() {
        let chosen = choose_sandbox_shell(
            Some("/bin/sh"),
            "/nix/store/missing-busybox/bin/busybox",
            Some(Path::new("/nix/store/def-busybox-static/bin/busybox")),
        );
        assert_eq!(chosen, "/nix/store/def-busybox-static/bin/busybox");
    }

    #[test]
    fn choose_sandbox_shell_keeps_existing_compile_default() {
        let dir = tempfile::tempdir().unwrap();
        let shell_path = dir.path().join("busybox");
        std::fs::write(&shell_path, "#!/bin/sh\n").unwrap();
        let chosen = choose_sandbox_shell(Some("/bin/sh"), shell_path.to_str().unwrap(), None);
        assert_eq!(chosen, shell_path.display().to_string());
    }

    #[test]
    fn choose_sandbox_shell_keeps_placeholder_without_static_candidate() {
        let chosen = choose_sandbox_shell(None, "/bin/sh", None);
        assert_eq!(chosen, "/bin/sh");
    }

    #[test]
    fn find_busybox_static_in_dir_finds_candidate() {
        let dir = tempfile::tempdir().unwrap();
        let candidate = dir.path().join("abc-busybox-static").join("bin");
        std::fs::create_dir_all(&candidate).unwrap();
        let busybox = candidate.join("busybox");
        std::fs::write(&busybox, "#!/bin/sh\n").unwrap();

        let found = find_busybox_static_in_dir(dir.path()).unwrap();
        assert_eq!(found, busybox);
    }

    #[test]
    fn should_materialize_inputs_fallback_matches_fusermount_exit() {
        let error =
            std::io::Error::other("fuse session failure: Unexpected exit code when running fusermount: Some(1)");
        assert!(should_materialize_inputs_fallback(&error));

        let unrelated = std::io::Error::other("failed to create build dir");
        assert!(!should_materialize_inputs_fallback(&unrelated));

        let wrong_kind =
            std::io::Error::new(ErrorKind::PermissionDenied, "Unexpected exit code when running fusermount: Some(1)");
        assert!(!should_materialize_inputs_fallback(&wrong_kind));
    }

    fn workspace_request(
        mode: crate::buildservice::StatefulWorkspaceMode,
    ) -> crate::buildservice::StatefulWorkspaceRequest {
        crate::buildservice::StatefulWorkspaceRequest {
            mode,
            workspace_id: Some("cargo-cache".to_string()),
            guest_path: PathBuf::from("/build/.mantle-workspace"),
            snapshot_input_name: Some(PathBuf::from("snapshot-root")),
            compatibility_digest_blake3: "a".repeat(TEST_DIGEST_HEX_LENGTH),
            toolchain_refs: Vec::new(),
            quota_bytes_max: 1,
            quota_files_max: 1,
            quota_snapshots_max: 1,
            retention_class: "recent".to_string(),
            retention_workspace_count_max: 1,
            retention_idle_generations_max: 1,
            retention_age_generations_max: 1,
            retention_quarantine_count_max: 1,
            generation: 1,
            lease: None,
            sensitive_paths: Vec::new(),
            secret_markers: Vec::new(),
            scan_depth_max: 1,
            path_bytes_max: 1,
            snapshot_enabled: false,
            clean_rebuild_enabled: false,
            clean_rebuild_require_declared_inputs: true,
            runtime_host_path: None,
        }
    }

    #[test]
    fn workspace_mounts_are_mode_specific_and_host_paths_stay_runtime_only() {
        let host = Path::new("/private/workdir");
        let immutable = workspace_mounts(
            host,
            Some(&workspace_request(crate::buildservice::StatefulWorkspaceMode::ImmutableSnapshot)),
        )
        .unwrap();
        assert_eq!(immutable[0].host_path, host.join("host_inputs_dir/snapshot-root"));
        assert!(immutable[0].read_only);
        assert_eq!(immutable[0].guest_path, PathBuf::from("/build/.mantle-workspace"));

        let mut mutable = workspace_request(crate::buildservice::StatefulWorkspaceMode::MutableSession);
        mutable.runtime_host_path = Some(PathBuf::from("/private/state/cargo-cache"));
        let mutable_mounts = workspace_mounts(host, Some(&mutable)).unwrap();
        assert_eq!(mutable_mounts[0].host_path, PathBuf::from("/private/state/cargo-cache"));
        assert!(!mutable_mounts[0].read_only);
        mutable.runtime_host_path = None;
        assert!(workspace_mounts(host, Some(&mutable)).is_err());
    }

    #[tokio::test]
    async fn materialize_inputs_blocking_supports_current_thread_runtime() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let file_node = insert_blob(&bs, b"current-thread", false).await;
        let root_nodes = BTreeMap::from([(PathComponent::try_from("input").unwrap(), file_node)]);
        let dest = tempfile::tempdir().unwrap();

        materialize_inputs_blocking(dest.path(), &root_nodes, &bs, &ds).unwrap();

        assert_eq!(std::fs::read(dest.path().join("input")).unwrap(), b"current-thread");
        assert!(dest.path().join("input").is_file());
    }

    #[tokio::test]
    async fn materialize_inputs_to_disk_writes_nested_tree() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let file_node = insert_blob(&bs, b"hello", true).await;
        let mut inner = Directory::new();
        inner.add("tool".try_into().unwrap(), file_node).unwrap();
        inner
            .add("tool-link".try_into().unwrap(), Node::Symlink {
                target: SymlinkTarget::try_from("tool").unwrap(),
            })
            .unwrap();
        let inner_digest = inner.digest();
        let inner_size = inner.size();
        ds.put(inner).await.unwrap();

        let mut root_nodes = BTreeMap::new();
        root_nodes.insert(PathComponent::try_from("pkg").unwrap(), Node::Directory {
            digest: inner_digest,
            size: inner_size,
        });

        let dest = tempfile::tempdir().unwrap();
        materialize_inputs_to_disk(dest.path(), &root_nodes, &bs, &ds).await.unwrap();

        let file_path = dest.path().join("pkg").join("tool");
        let link_path = dest.path().join("pkg").join("tool-link");
        assert_eq!(std::fs::read(&file_path).unwrap(), b"hello");
        let link_target = std::fs::read_link(&link_path).unwrap();
        assert_eq!(link_target, PathBuf::from("tool"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file_path).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0, "materialized executable should keep x bit");
        }
    }

    fn watch_build_request(marker_host: &Path) -> BuildRequest {
        let mut workspace = workspace_request(crate::buildservice::StatefulWorkspaceMode::MutableSession);
        workspace.guest_path = PathBuf::from("/build/markers");
        workspace.runtime_host_path = Some(marker_host.to_path_buf());
        BuildRequest {
            working_dir: PathBuf::from("build"),
            scratch_paths: vec![PathBuf::from("build"), PathBuf::from("nix/store")],
            inputs_dir: PathBuf::from("nix/store"),
            outputs: vec![PathBuf::from("nix/store/watch-result")],
            constraints: std::iter::once(BuildConstraints::ProvideBinSh).collect(),
            command_args: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "/bin/busybox setsid /bin/sh -c '/bin/busybox echo escaped > /build/markers/escaped; /bin/busybox sleep 2; /bin/busybox echo late > /build/markers/late' & /bin/busybox echo started > /build/markers/started; wait; /bin/busybox echo done > /nix/store/watch-result".to_string(),
            ],
            workspace: Some(workspace),
            ..Default::default()
        }
    }

    async fn await_watch_start(marker_host: &Path) {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        while !marker_host.join("started").is_file() || !marker_host.join("escaped").is_file() {
            assert!(tokio::time::Instant::now() < deadline, "real BuildService sandbox did not start");
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    #[tokio::test]
    async fn real_buildservice_watch_cancellation_reaps_pid1_and_blocks_late_output() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        let request = watch_build_request(markers.path());
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let build = tokio::spawn(async move { service.do_build_cancellable(request, cancel_rx).await });
        await_watch_start(markers.path()).await;
        cancel_tx.send(true).unwrap();
        let error = build.await.unwrap().unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Interrupted, "only observed teardown is cancellable: {error}");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(!markers.path().join("late").exists(), "BuildService cancellation allowed late descendant output");
    }

    #[tokio::test]
    async fn queued_watch_cancellation_never_starts_a_sandbox() {
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let mut service =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        service.concurrent_builds = std::sync::Arc::new(tokio::sync::Semaphore::new(0));
        let service = std::sync::Arc::new(service);
        let admitted = std::sync::Arc::clone(&service);
        let request = watch_build_request(markers.path());
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let waiting = tokio::spawn(async move { admitted.do_build_cancellable(request, cancel_rx).await });
        tokio::task::yield_now().await;
        cancel_tx.send(true).unwrap();
        service.concurrent_builds.add_permits(1);
        let error = tokio::time::timeout(std::time::Duration::from_secs(5), waiting)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Interrupted);
        assert!(!markers.path().join("started").exists(), "queued cancellation spawned a builder");
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none(), "queued cancellation allocated a sandbox");
    }

    #[tokio::test]
    async fn pidfd_preflight_failure_does_not_spawn_a_builder() {
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let mut service =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        service.watch_fault = Some(WatchFault::PidfdUnavailable);
        let (_keep_sender, cancel_rx) = watch::channel(false);
        let error = service.do_build_cancellable(watch_build_request(markers.path()), cancel_rx).await.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert!(!markers.path().join("started").exists(), "unavailable pidfd started a builder");
        assert!(!markers.path().join("late").exists(), "unavailable pidfd left a writer");
    }

    #[tokio::test]
    async fn real_bwrap_early_failure_is_terminal_before_pid1_without_leaking_admission() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let mut service =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        service.watch_fault = Some(WatchFault::ExitBeforePid1);
        let (_keep_sender, cancel_rx) = watch::channel(false);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            service.do_build_cancellable(watch_build_request(markers.path()), cancel_rx),
        )
        .await
        .unwrap();
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("exited before watch PID1 ownership"),
            "early failure was not classified: {error}"
        );
        assert!(
            error.to_string().contains("Unknown option --invalid-bwrap-watch-early-exit"),
            "real bwrap parse failure was lost: {error}"
        );
        assert_eq!(service.concurrent_builds.available_permits(), 2, "terminal startup held admission");
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none(), "terminal startup leaked workspace");
        assert!(!markers.path().join("started").exists(), "invalid bubblewrap started a builder");
        assert!(!markers.path().join("late").exists(), "invalid bubblewrap left a late builder");
    }

    #[tokio::test]
    async fn supervisor_panic_after_pid1_stops_gated_builder_before_returning_error() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let mut service =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        service.watch_fault = Some(WatchFault::PanicAfterPid1);
        let (_keep_sender, cancel_rx) = watch::channel(false);
        let error = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            service.do_build_cancellable(watch_build_request(markers.path()), cancel_rx),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert!(
            error.to_string().contains("supervisor failed after owned teardown"),
            "panic escaped teardown: {error}"
        );
        assert_eq!(service.concurrent_builds.available_permits(), 2, "supervisor panic retained admission");
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none(), "supervisor panic leaked workspace");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(!markers.path().join("started").exists(), "gate EOF started payload after supervisor panic");
        assert!(!markers.path().join("late").exists(), "supervisor panic left a late builder");
    }

    #[tokio::test]
    async fn aborting_caller_then_panicking_supervisor_retains_owned_teardown() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let mut service =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        service.watch_fault = Some(WatchFault::PanicAfterAbort);
        let service = std::sync::Arc::new(service);
        let observing = std::sync::Arc::clone(&service);
        let (_keep_sender, cancel_rx) = watch::channel(false);
        let request = watch_build_request(markers.path());
        let build = tokio::spawn(async move { service.do_build_cancellable(request, cancel_rx).await });
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let owned = std::fs::read_dir(root.path())
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry.path().join("watch-pid1-owned").is_file());
            if owned {
                break;
            }
            assert!(tokio::time::Instant::now() < deadline, "supervisor did not own PID1 before abort");
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(observing.concurrent_builds.available_permits(), 1, "fault never admitted a sandbox");
        build.abort();
        assert!(build.await.unwrap_err().is_cancelled());
        let _permit =
            tokio::time::timeout(std::time::Duration::from_secs(5), observing.concurrent_builds.acquire_many(2))
                .await
                .unwrap()
                .unwrap();
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none(), "aborted panic leaked workspace");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(!markers.path().join("started").exists(), "aborted supervisor panic released gate EOF");
        assert!(!markers.path().join("late").exists(), "aborted supervisor panic left a writer");
    }

    #[tokio::test]
    async fn delayed_pid1_discovery_keeps_payload_gated_and_cancels_before_start() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let blobs = MemoryBlobService::default();
        let mut service = BubblewrapBuildService::new(root.path().to_path_buf(), blobs.clone(), tmp_ds());
        service.watch_fault = Some(WatchFault::DelayPid1Discovery);
        service.concurrent_builds = std::sync::Arc::new(tokio::sync::Semaphore::new(1));
        let service = std::sync::Arc::new(service);
        let observing = std::sync::Arc::clone(&service);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let request = watch_build_request(markers.path());
        let build = tokio::spawn(async move { service.do_build_cancellable(request, cancel_rx).await });
        let sibling_markers = tempfile::tempdir().unwrap();
        let sibling_request = watch_build_request(sibling_markers.path());
        let sibling_service = std::sync::Arc::clone(&observing);
        let sibling = tokio::spawn(async move { sibling_service.do_build(sibling_request).await });
        tokio::time::sleep(std::time::Duration::from_millis(5200)).await;
        assert!(!markers.path().join("started").exists(), "payload ran without owned PID1");
        assert!(!markers.path().join("late").exists(), "postspawn discovery failure lost the start gate");
        assert!(!sibling_markers.path().join("started").exists(), "sibling started while PID1 ownership was unknown");
        cancel_tx.send(true).unwrap();
        assert_eq!(observing.concurrent_builds.available_permits(), 0, "unobserved PID1 teardown released admission");
        assert!(
            std::fs::read_dir(root.path()).unwrap().next().is_some(),
            "unobserved teardown removed the workspace"
        );
        let error = tokio::time::timeout(std::time::Duration::from_secs(5), build).await.unwrap().unwrap().unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Interrupted, "delayed discovery released a retryable failure: {error}");
        let sibling_result =
            tokio::time::timeout(std::time::Duration::from_secs(5), sibling).await.unwrap().unwrap().unwrap();
        let Node::File { digest, .. } = &sibling_result.outputs[0].node else {
            panic!("sibling did not publish completed output");
        };
        let mut output = Vec::new();
        blobs.open_read(digest).await.unwrap().unwrap().read_to_end(&mut output).await.unwrap();
        assert_eq!(output, b"done\n", "sibling output was admitted before teardown");
        assert_eq!(observing.concurrent_builds.available_permits(), 1, "observed teardown held admission");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(sibling_markers.path().join("started").exists(), "sibling never started after observed teardown");
        assert!(!markers.path().join("late").exists(), "cancelled gated builder ran later");
    }

    #[tokio::test]
    async fn dropping_cancellable_build_future_reaps_escaped_child() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        let (_keep_sender, cancel_rx) = watch::channel(false);
        let request = watch_build_request(markers.path());
        let build = tokio::spawn(async move { service.do_build_cancellable(request, cancel_rx).await });
        await_watch_start(markers.path()).await;
        build.abort();
        assert!(build.await.unwrap_err().is_cancelled());
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(!markers.path().join("late").exists(), "dropped cancellable future allowed escaped builder output");
    }

    #[tokio::test]
    async fn successful_watch_build_ingests_exact_output_bytes() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let blobs = MemoryBlobService::default();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), blobs.clone(), tmp_ds());
        let (_keep_sender, cancel_rx) = watch::channel(false);
        let result = service.do_build_cancellable(watch_build_request(markers.path()), cancel_rx).await.unwrap();
        let Node::File { digest, .. } = &result.outputs[0].node else {
            panic!("watch build did not publish a file");
        };
        let mut output = Vec::new();
        blobs.open_read(digest).await.unwrap().unwrap().read_to_end(&mut output).await.unwrap();
        assert_eq!(output, b"done\n");
        assert_eq!(std::fs::read(markers.path().join("late")).unwrap(), b"late\n");
    }

    #[tokio::test]
    async fn closing_cancellation_sender_reaps_escaped_child() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        let (sender, cancel_rx) = watch::channel(false);
        let request = watch_build_request(markers.path());
        let build = tokio::spawn(async move { service.do_build_cancellable(request, cancel_rx).await });
        await_watch_start(markers.path()).await;
        drop(sender);
        let error = tokio::time::timeout(std::time::Duration::from_secs(5), build).await.unwrap().unwrap().unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Interrupted, "lost cancellation owner permitted output: {error}");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(!markers.path().join("late").exists(), "lost cancellation owner allowed escaped builder output");
    }

    #[tokio::test]
    async fn unchanged_buildservice_path_still_ingests_completed_sandbox_output() {
        assert!(Path::new(&sandbox_shell()).is_file(), "real BuildService needs a static shell");
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let blobs = MemoryBlobService::default();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), blobs.clone(), tmp_ds());
        let result = service.do_build(watch_build_request(markers.path())).await.unwrap();
        let Node::File { digest, .. } = &result.outputs[0].node else {
            panic!("real completed sandbox did not publish a file");
        };
        let mut output = Vec::new();
        blobs.open_read(digest).await.unwrap().unwrap().read_to_end(&mut output).await.unwrap();
        assert_eq!(output, b"done\n");
        assert_eq!(std::fs::read(markers.path().join("late")).unwrap(), b"late\n");
        assert!(result.timing.is_some(), "local sandbox result must report separate admission and execution timing");
    }

    #[test]
    fn sandbox_concurrency_policy_respects_jobs_and_fd_budget_bounds() {
        assert_eq!(SandboxConcurrencyPolicy::for_jobs(1).effective_permits().unwrap(), 1);
        assert_eq!(SandboxConcurrencyPolicy::for_jobs(2).effective_permits().unwrap(), 2);
        assert_eq!(SandboxConcurrencyPolicy::for_jobs(4).effective_permits().unwrap(), 4);
        assert_eq!(
            SandboxConcurrencyPolicy::for_jobs(MAX_CONCURRENT_SANDBOX_BUILDS + 16).effective_permits().unwrap(),
            MAX_CONCURRENT_SANDBOX_BUILDS
        );
        assert_eq!(
            SandboxConcurrencyPolicy {
                requested_jobs: 8,
                fd_budget: ESTIMATED_FDS_PER_SANDBOX * 2,
            }
            .effective_permits()
            .unwrap(),
            2
        );

        let zero_err = SandboxConcurrencyPolicy::for_jobs(0).effective_permits().unwrap_err();
        assert_eq!(zero_err.kind(), ErrorKind::InvalidInput);
        let low_fd_err = SandboxConcurrencyPolicy {
            requested_jobs: 2,
            fd_budget: ESTIMATED_FDS_PER_SANDBOX - 1,
        }
        .effective_permits()
        .unwrap_err();
        assert_eq!(low_fd_err.kind(), ErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn four_job_sandbox_policy_overlaps_four_independent_sandboxes_and_fd_cap_queues_excess() {
        if !Path::new(&sandbox_shell()).is_file() {
            eprintln!("skipping: static sandbox shell unavailable");
            return;
        }
        let bwrap_ok = std::process::Command::new("bwrap")
            .args(["--ro-bind", "/", "/", "--", "/bin/true"])
            .status()
            .is_ok_and(|s| s.success());
        if !bwrap_ok {
            eprintln!("skipping: bwrap unavailable");
            return;
        }

        let make_sleep_request = |id: u32| -> BuildRequest {
            BuildRequest {
                working_dir: PathBuf::from("build"),
                scratch_paths: vec![PathBuf::from("build"), PathBuf::from("nix/store")],
                inputs_dir: PathBuf::from("nix/store"),
                outputs: vec![PathBuf::from(format!("nix/store/out-{id}"))],
                constraints: std::iter::once(BuildConstraints::ProvideBinSh).collect(),
                command_args: vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    format!("/bin/busybox sleep 1; /bin/busybox echo ok-{id} > /nix/store/out-{id}"),
                ],
                ..Default::default()
            }
        };

        // 1. With --jobs 4, four independent 1-second sandboxes overlap concurrently.
        let root = tempfile::tempdir().unwrap();
        let blobs = MemoryBlobService::default();
        let service = Arc::new(
            BubblewrapBuildService::new(root.path().to_path_buf(), blobs, tmp_ds())
                .with_max_concurrent_builds(4)
                .unwrap(),
        );
        assert_eq!(service.max_concurrent_builds(), 4);

        let mut handles = Vec::new();
        for id in 0..4u32 {
            let svc = Arc::clone(&service);
            let req = make_sleep_request(id);
            handles.push(tokio::spawn(async move {
                let started = std::time::Instant::now();
                let result = svc.do_build(req).await.unwrap();
                let finished = std::time::Instant::now();
                (started, finished, result)
            }));
        }
        let mut intervals = Vec::new();
        for handle in handles {
            let (started, finished, result) = handle.await.unwrap();
            let timing = result.timing.expect("local sandbox must report timing");
            assert!(timing.execution_ms >= 500, "execution_ms should reflect ~1s sleep: {timing:?}");
            intervals.push((started, finished));
        }
        let latest_start = intervals.iter().map(|(s, _)| *s).max().unwrap();
        let earliest_finish = intervals.iter().map(|(_, f)| *f).min().unwrap();
        assert!(
            latest_start < earliest_finish,
            "all 4 sandboxes must overlap concurrently under jobs=4 (latest_start={latest_start:?}, earliest_finish={earliest_finish:?})"
        );

        // 2. When FD budget caps concurrency to 2, 4 builds queue cleanly and report admission wait.
        let root_cap = tempfile::tempdir().unwrap();
        let blobs_cap = MemoryBlobService::default();
        let capped = Arc::new(
            BubblewrapBuildService::new(root_cap.path().to_path_buf(), blobs_cap, tmp_ds())
                .with_concurrency_policy(SandboxConcurrencyPolicy {
                    requested_jobs: 4,
                    fd_budget: ESTIMATED_FDS_PER_SANDBOX * 2,
                })
                .unwrap(),
        );
        assert_eq!(capped.max_concurrent_builds(), 2);

        let mut capped_handles = Vec::new();
        for id in 0..4u32 {
            let svc = Arc::clone(&capped);
            let req = make_sleep_request(id);
            capped_handles.push(tokio::spawn(async move { svc.do_build(req).await.unwrap() }));
        }
        let mut max_wait_ms = 0u64;
        for handle in capped_handles {
            let result = handle.await.unwrap();
            let timing = result.timing.expect("capped sandbox must report timing");
            max_wait_ms = max_wait_ms.max(timing.admission_wait_ms);
        }
        assert!(
            max_wait_ms >= 400,
            "FD-capped 2-slot queue must record distinct admission_wait_ms for queued builds, got max_wait_ms={max_wait_ms}"
        );
    }

    fn shell_build_request(script: &str, outputs: &[&str]) -> BuildRequest {
        BuildRequest {
            working_dir: PathBuf::from("build"),
            scratch_paths: vec![PathBuf::from("build"), PathBuf::from("nix/store")],
            inputs_dir: PathBuf::from("nix/store"),
            outputs: outputs.iter().map(PathBuf::from).collect(),
            constraints: std::iter::once(BuildConstraints::ProvideBinSh).collect(),
            command_args: vec!["/bin/sh".to_string(), "-c".to_string(), script.to_string()],
            ..Default::default()
        }
    }

    fn require_real_bwrap_and_shell() -> bool {
        if !Path::new(&sandbox_shell()).is_file() {
            eprintln!("skipping: static sandbox shell unavailable");
            return false;
        }
        let bwrap_ok = std::process::Command::new(crate::bwrap::bwrap_program())
            .args(["--ro-bind", "/", "/", "--", "/bin/true"])
            .status()
            .is_ok_and(|s| s.success());
        if !bwrap_ok {
            eprintln!("skipping: bwrap unavailable");
            return false;
        }
        true
    }

    #[tokio::test]
    async fn wall_deadline_kills_hung_sandbox_and_setsid_descendant_without_admitting_output() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let blobs = MemoryBlobService::default();
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.wall_deadline = crate::buildservice::SandboxWallDeadline::Bounded { wall_deadline_ms: 250 };
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), blobs.clone(), tmp_ds())
            .with_execution_policy(policy)
            .unwrap();
        let mut request = shell_build_request(
            "/bin/busybox setsid /bin/sh -c '/bin/busybox sleep 1; /bin/busybox echo late > /build/markers/late' & /bin/busybox echo started > /build/markers/started; /bin/busybox printf 'partial-wall-output\\n' > /nix/store/wall-out; /bin/busybox sleep 30",
            &["nix/store/wall-out"],
        );
        let partial_digest: snix_castore::B3Digest = blake3::hash(b"partial-wall-output\n").into();
        let mut workspace = workspace_request(crate::buildservice::StatefulWorkspaceMode::MutableSession);
        workspace.guest_path = PathBuf::from("/build/markers");
        workspace.runtime_host_path = Some(markers.path().to_path_buf());
        request.workspace = Some(workspace);

        let err = service.do_build(request).await.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::TimedOut);
        let classified = err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .expect("wall deadline error must carry ClassifiedBuildServiceError");
        assert_eq!(classified.class, crate::buildservice::SandboxFailureClass::WallDeadlineExceeded);
        assert!(
            !blobs.has(&partial_digest).await.unwrap(),
            "wall timeout must not ingest pre-written partial output"
        );
        assert!(blobs.open_read(&partial_digest).await.unwrap().is_none());
        assert_eq!(service.concurrent_builds.available_permits(), 2);
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none(), "wall timeout leaked sandbox dir");
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        assert!(!markers.path().join("late").exists(), "wall timeout left orphan setsid descendant");
    }

    #[tokio::test]
    async fn silent_but_valid_process_passes_under_default_policy_while_silence_budget_kills_hang() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        // 1. Default policy disables silence budget so silent valid builds succeed.
        let root_default = tempfile::tempdir().unwrap();
        let blobs_default = MemoryBlobService::default();
        let default_svc =
            BubblewrapBuildService::new(root_default.path().to_path_buf(), blobs_default.clone(), tmp_ds());
        assert_eq!(default_svc.execution_policy().silence_budget, crate::buildservice::SandboxSilenceBudget::Disabled);
        let silent_req =
            shell_build_request("/bin/busybox sleep 1; /bin/busybox echo silent-ok > /nix/store/silent-out", &[
                "nix/store/silent-out",
            ]);
        let res = default_svc.do_build(silent_req).await.unwrap();
        assert_eq!(res.outputs.len(), 1);

        // 2. Explicit silence budget expires when output goes silent.
        let root_silence = tempfile::tempdir().unwrap();
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.wall_deadline = crate::buildservice::SandboxWallDeadline::Bounded {
            wall_deadline_ms: 10_000,
        };
        policy.silence_budget = crate::buildservice::SandboxSilenceBudget::Bounded {
            silence_timeout_ms: 300,
        };
        let silence_svc =
            BubblewrapBuildService::new(root_silence.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_execution_policy(policy)
                .unwrap();
        let stalled_req = shell_build_request(
            "/bin/busybox echo initial-byte; /bin/busybox sleep 10; /bin/busybox echo done > /nix/store/stalled-out",
            &["nix/store/stalled-out"],
        );
        let err = silence_svc.do_build(stalled_req).await.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::TimedOut);
        let classified = err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .expect("silence timeout must carry ClassifiedBuildServiceError");
        assert_eq!(classified.class, crate::buildservice::SandboxFailureClass::SilenceBudgetExceeded);
    }

    #[tokio::test]
    async fn noisy_build_past_capture_truncation_keeps_resetting_silence_budget() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.wall_deadline = crate::buildservice::SandboxWallDeadline::Bounded {
            wall_deadline_ms: 15_000,
        };
        policy.silence_budget = crate::buildservice::SandboxSilenceBudget::Bounded {
            silence_timeout_ms: 450,
        };
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(policy)
            .unwrap();
        // Emit >512 KiB (> 480 KiB truncation limit) in bursts spaced by 150ms (> 450ms total).
        let script = "i=0; while [ \"$i\" -lt 5 ]; do /bin/busybox dd if=/dev/zero bs=131072 count=1 2>/dev/null | /bin/busybox tr '\\0' 'a'; /bin/busybox usleep 150000; i=$((i+1)); done; /bin/busybox echo noisy-ok > /nix/store/noisy-out";
        let req = shell_build_request(script, &["nix/store/noisy-out"]);
        let result = service
            .do_build(req)
            .await
            .expect("noisy long-running build past truncation must not trigger silence timeout");
        assert_eq!(result.outputs.len(), 1);
        assert!(
            result.log.as_deref().is_some_and(|l| l.contains("sandbox output truncated")),
            "expected truncated log marker"
        );
    }

    #[tokio::test]
    async fn unlimited_wall_deadline_opt_out_completes_cleanly() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(crate::buildservice::SandboxExecutionPolicy::unlimited())
            .unwrap();
        let req = shell_build_request("/bin/busybox echo unlimited-ok > /nix/store/unlimited-out", &[
            "nix/store/unlimited-out",
        ]);
        let result = service.do_build(req).await.unwrap();
        assert_eq!(result.outputs.len(), 1);
    }

    #[tokio::test]
    async fn dropping_one_shot_do_build_future_reaps_setsid_descendant_and_releases_permit() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let service = Arc::new(
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_max_concurrent_builds(1)
                .unwrap(),
        );
        let svc_clone = Arc::clone(&service);
        let request = watch_build_request(markers.path());
        let build = tokio::spawn(async move { svc_clone.do_build(request).await });
        await_watch_start(markers.path()).await;
        build.abort();
        assert!(build.await.unwrap_err().is_cancelled());

        // Permit and workspace must be released after PID1 teardown.
        let _permit = tokio::time::timeout(std::time::Duration::from_secs(5), service.concurrent_builds.acquire())
            .await
            .unwrap()
            .unwrap();
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none(), "aborted one-shot build leaked workspace");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        assert!(!markers.path().join("late").exists(), "dropped one-shot future allowed escaped descendant output");
    }

    #[tokio::test]
    async fn external_shutdown_grace_period_kills_sigterm_ignoring_leaking_descendant() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let markers = tempfile::tempdir().unwrap();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.shutdown_grace_ms = 200;
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(policy)
            .unwrap()
            .with_shutdown_signal(shutdown_rx);

        let mut request = shell_build_request(
            "trap '' TERM; /bin/busybox setsid /bin/sh -c '/bin/busybox echo escaped > /build/markers/escaped; /bin/busybox sleep 2; /bin/busybox echo late > /build/markers/late' & /bin/busybox echo started > /build/markers/started; /bin/busybox sleep 10; /bin/busybox echo done > /nix/store/out",
            &["nix/store/out"],
        );
        let mut workspace = workspace_request(crate::buildservice::StatefulWorkspaceMode::MutableSession);
        workspace.guest_path = PathBuf::from("/build/markers");
        workspace.runtime_host_path = Some(markers.path().to_path_buf());
        request.workspace = Some(workspace);

        let handle = tokio::spawn(async move { service.do_build(request).await });
        await_watch_start(markers.path()).await;
        shutdown_tx.send(true).unwrap();

        let err = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await.unwrap().unwrap().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Interrupted);
        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
        assert!(!markers.path().join("late").exists(), "leaking descendant survived finite shutdown grace");
    }

    #[tokio::test]
    async fn resource_enforcement_negative_controls_report_unavailable_and_ignore_min_memory() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        // 1. BuildConstraints::MinMemory alone does NOT enable or fake hard cgroup enforcement.
        let root = tempfile::tempdir().unwrap();
        let service = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds());
        let mut req = shell_build_request("/bin/busybox echo ok > /nix/store/out", &["nix/store/out"]);
        req.constraints.insert(BuildConstraints::MinMemory(1024));
        service.do_build(req).await.unwrap();
        let facts = service.last_enforcement_facts().unwrap();
        assert!(!facts.enforced_memory, "MinMemory constraint must never claim hard memory enforcement");

        // 2. Non-cgroup2 directory fails closed under Required mode.
        let fake_cg = tempfile::tempdir().unwrap();
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.resources.cgroup_root = Some(fake_cg.path().to_path_buf());
        policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::Required;
        policy.resources.memory_limit_bytes = Some(32 * 1024 * 1024);
        let strict_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(policy.clone())
            .unwrap();
        let err = strict_svc
            .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out", &["nix/store/out"]))
            .await
            .unwrap_err();
        let classified = err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .unwrap();
        assert_eq!(classified.class, crate::buildservice::SandboxFailureClass::ResourceEnforcementUnavailable);

        // 3. BestEffort with unavailable cgroup runs honestly without claiming enforcement.
        policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::BestEffort;
        let best_effort_svc =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_execution_policy(policy)
                .unwrap();
        best_effort_svc
            .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out", &["nix/store/out"]))
            .await
            .unwrap();
        let be_facts = best_effort_svc.last_enforcement_facts().unwrap();
        assert!(!be_facts.enforced_memory);
        assert!(!be_facts.unavailable_reasons.is_empty());

        // 4. Raw SSH session targeting delegated user service slice fails LCA check honestly.
        if let Ok(self_cg) = std::fs::read_to_string("/proc/self/cgroup")
            && let Some(rel) = self_cg.lines().find_map(|l| l.strip_prefix("0::"))
            && rel.contains("/session-")
        {
            let euid = unsafe { nix::libc::geteuid() };
            let user_svc_cg = PathBuf::from(format!("/sys/fs/cgroup/user.slice/user-{euid}.slice/user@{euid}.service"));
            if user_svc_cg.is_dir() {
                let mut lca_policy = crate::buildservice::SandboxExecutionPolicy::standard();
                lca_policy.resources.cgroup_root = Some(user_svc_cg);
                lca_policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::Required;
                lca_policy.resources.memory_limit_bytes = Some(32 * 1024 * 1024);
                let lca_svc =
                    BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                        .with_execution_policy(lca_policy)
                        .unwrap();
                let lca_err = lca_svc
                    .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out", &["nix/store/out"]))
                    .await
                    .unwrap_err();
                let lca_facts = lca_svc.last_enforcement_facts().unwrap();
                assert!(lca_facts.unavailable_reasons.iter().any(|r| matches!(
                    r,
                    crate::buildservice::SandboxUnavailableReason::DelegationLcaNotWritable { .. }
                )));
                let classified = lca_err
                    .get_ref()
                    .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
                    .unwrap();
                assert_eq!(classified.class, crate::buildservice::SandboxFailureClass::ResourceEnforcementUnavailable);
            }
        }
    }

    fn resolve_or_spawn_delegated_cgroup_root(test_fn_name: &str) -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("MANTLE_TEST_CGROUP_ROOT") {
            return Some(PathBuf::from(path));
        }
        // If already running inside a delegated scope (with memory & pids), prepare child root.
        if let Ok(self_cg) = std::fs::read_to_string("/proc/self/cgroup")
            && let Some(rel) = self_cg.lines().find_map(|l| l.strip_prefix("0::"))
        {
            let cur_dir = PathBuf::from(format!("/sys/fs/cgroup{}", rel.trim()));
            let controllers = std::fs::read_to_string(cur_dir.join("cgroup.controllers")).unwrap_or_default();
            if controllers.contains("memory")
                && controllers.contains("pids")
                && !rel.contains("/session-")
                && std::env::var_os("MANTLE_DELEGATED_CHILD").is_some()
            {
                let runner_dir = cur_dir.join("test-runner");
                let sandbox_root = cur_dir.join("sandbox-root");
                let _ = std::fs::create_dir(&runner_dir);
                let _ = std::fs::create_dir(&sandbox_root);
                let _ = std::fs::write(runner_dir.join("cgroup.procs"), std::process::id().to_string());
                let _ = std::fs::write(cur_dir.join("cgroup.subtree_control"), "+memory +pids");
                return Some(sandbox_root);
            }
        }
        if std::env::var_os("MANTLE_DELEGATED_CHILD").is_some() {
            return None;
        }
        // Re-exec this exact test inside systemd-run --user --scope --property=Delegate=yes.
        let euid = unsafe { nix::libc::geteuid() };
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("/run/user/{euid}")));
        if !runtime_dir.join("bus").exists() {
            eprintln!("skipping delegated cgroup test: user systemd D-Bus unavailable");
            return None;
        }
        let exe = std::env::current_exe().ok()?;
        let mut cmd = std::process::Command::new("systemd-run");
        cmd.env("XDG_RUNTIME_DIR", &runtime_dir)
            .env("MANTLE_DELEGATED_CHILD", "1")
            .args(["--user", "--scope", "--property=Delegate=yes", "--quiet", "--"])
            .arg(exe)
            .arg("--exact")
            .arg(test_fn_name)
            .arg("--nocapture");
        if let Some(bwrap) = std::env::var_os("SNIX_BUILD_BWRAP") {
            cmd.env("SNIX_BUILD_BWRAP", bwrap);
        }
        if let Some(shell) = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL") {
            cmd.env("SNIX_BUILD_SANDBOX_SHELL", shell);
        }
        if let Some(no_fuse) = std::env::var_os("CRUNCH_NO_FUSE") {
            cmd.env("CRUNCH_NO_FUSE", no_fuse);
        }
        let output = match cmd.output() {
            Ok(out) => out,
            Err(err) => {
                eprintln!("skipping delegated cgroup test: systemd-run failed to launch: {err}");
                return None;
            }
        };
        assert!(
            output.status.success(),
            "delegated cgroup test failed inside systemd-run:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }

    #[tokio::test]
    async fn delegated_cgroup_hard_memory_limit_under_limit_oom_and_helper_scope_controls() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let Some(cg_root) = resolve_or_spawn_delegated_cgroup_root(
            "buildservice::bwrap::tests::delegated_cgroup_hard_memory_limit_under_limit_oom_and_helper_scope_controls",
        ) else {
            return;
        };

        let root = tempfile::tempdir().unwrap();

        // 1. Under-limit healthy control: allocates 2 MiB under 64 MiB hard cap, records membership &
        //    limits.
        let mut under_policy = crate::buildservice::SandboxExecutionPolicy::standard();
        under_policy.resources.cgroup_root = Some(cg_root.clone());
        under_policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::Required;
        under_policy.resources.memory_limit_bytes = Some(64 * 1024 * 1024);
        under_policy.resources.swap_policy = crate::buildservice::SandboxSwapPolicy::Zero;
        under_policy.resources.pids_mode = crate::buildservice::SandboxEnforcementMode::Required;
        under_policy.resources.pids_limit_count = Some(32);
        let under_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(under_policy)
            .unwrap();
        let under_req = shell_build_request(
            "/bin/busybox dd if=/dev/zero of=/tmp/small bs=1M count=2 2>/dev/null && /bin/busybox echo ok > /nix/store/under-out",
            &["nix/store/under-out"],
        );
        under_svc.do_build(under_req).await.expect("under-limit control must pass");
        let under_facts = under_svc.last_enforcement_facts().unwrap();
        assert!(under_facts.enforced_memory);
        assert!(under_facts.enforced_pids);
        assert!(under_facts.populated_observed);
        assert!(!under_facts.observed_member_pids.is_empty());
        assert_eq!(
            under_facts.effective_memory_max,
            crate::buildservice::SandboxEffectiveLimit::Bytes(64 * 1024 * 1024)
        );
        assert_eq!(under_facts.effective_swap_max, crate::buildservice::SandboxEffectiveLimit::Bytes(0));
        assert_eq!(under_facts.effective_oom_group, Some(true));
        assert_eq!(under_facts.effective_pids_max, crate::buildservice::SandboxEffectiveLimit::Count(32));
        assert_eq!(under_facts.oom_kill_count, 0);

        // 2. Over-limit child/grandchild touching allocated pages -> OOM kill & ResourceBudgetExceeded.
        let mut over_policy = crate::buildservice::SandboxExecutionPolicy::standard();
        over_policy.resources.cgroup_root = Some(cg_root.clone());
        over_policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::Required;
        over_policy.resources.memory_limit_bytes = Some(16 * 1024 * 1024);
        over_policy.resources.swap_policy = crate::buildservice::SandboxSwapPolicy::Zero;
        let over_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(over_policy.clone())
            .unwrap();
        let over_req = shell_build_request(
            "/bin/busybox sh -c '/bin/busybox dd if=/dev/zero of=/tmp/memhog bs=1M count=40 2>/dev/null' && /bin/busybox echo unexpected > /nix/store/over-out",
            &["nix/store/over-out"],
        );
        let err = over_svc.do_build(over_req).await.unwrap_err();
        let classified = err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .expect("OOM must produce ClassifiedBuildServiceError");
        assert_eq!(classified.class, crate::buildservice::SandboxFailureClass::ResourceBudgetExceeded);
        let over_facts = over_svc.last_enforcement_facts().unwrap();
        assert!(over_facts.oom_kill_count > 0, "expected kernel oom_kill > 0, got {over_facts:?}");
        assert!(over_facts.memory_peak_bytes.is_some());

        // 3. Helper daemon running outside enforced cgroup -> refused before payload start!
        let sock_dir = tempfile::tempdir().unwrap();
        let compile_sock = sock_dir.path().join("compile.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&compile_sock).unwrap();
        let helper_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(over_policy)
            .unwrap()
            .with_compile_socket(compile_sock)
            .unwrap();
        let helper_err = helper_svc
            .do_build(shell_build_request("/bin/busybox echo nope > /nix/store/out", &["nix/store/out"]))
            .await
            .unwrap_err();
        let helper_classified = helper_err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .unwrap();
        assert_eq!(helper_classified.class, crate::buildservice::SandboxFailureClass::ResourceEnforcementUnavailable);
        let helper_facts = helper_svc.last_enforcement_facts().unwrap();
        assert!(helper_facts.unavailable_reasons.iter().any(|r| matches!(
            r,
            crate::buildservice::SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { .. }
        )));
    }

    #[tokio::test]
    async fn delegated_cgroup_dgram_helper_scope_and_pids_limit_controls() {
        use std::io::BufRead;
        use std::io::BufReader;
        use std::io::Read as _;
        use std::io::Write as _;
        use std::os::unix::net::UnixDatagram;
        use std::process::Stdio;

        if let Some(sock_path) = std::env::var_os("MANTLE_TEST_EXTERNAL_DGRAM_SOCKET") {
            let _sock = UnixDatagram::bind(&sock_path).expect("external child must bind dgram socket");
            println!("READY");
            let _ = std::io::stdout().flush();
            let mut stdin_buf = [0_u8; 16];
            while let Ok(n) = std::io::stdin().read(&mut stdin_buf) {
                if n == 0 {
                    break;
                }
            }
            return;
        }

        if !require_real_bwrap_and_shell() {
            return;
        }
        let Some(cg_root) = resolve_or_spawn_delegated_cgroup_root(
            "buildservice::bwrap::tests::delegated_cgroup_dgram_helper_scope_and_pids_limit_controls",
        ) else {
            return;
        };

        let root = tempfile::tempdir().unwrap();
        let sock_dir = tempfile::tempdir().unwrap();
        let in_proc_wait_sock = sock_dir.path().join("rust-waits.sock");
        let _in_proc_sock = UnixDatagram::bind(&in_proc_wait_sock).unwrap();

        // 1. In-process UnixDatagram + AllowInProcessPassiveDatagram -> passes & records outcome.
        let mut allow_policy = crate::buildservice::SandboxExecutionPolicy::standard();
        allow_policy.resources.cgroup_root = Some(cg_root.clone());
        allow_policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::Required;
        allow_policy.resources.memory_limit_bytes = Some(64 * 1024 * 1024);
        allow_policy.resources.swap_policy = crate::buildservice::SandboxSwapPolicy::Zero;
        allow_policy.resources.pids_mode = crate::buildservice::SandboxEnforcementMode::Required;
        allow_policy.resources.pids_limit_count = Some(32);
        allow_policy.resources.helper_scope_policy =
            crate::buildservice::SandboxHelperScopePolicy::AllowInProcessPassiveDatagram;

        let allow_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(allow_policy.clone())
            .unwrap()
            .with_rust_wait_socket(in_proc_wait_sock.clone())
            .unwrap();
        let ok_req = shell_build_request(
            "test -S /run/mantle/rust-waits.sock && /bin/busybox echo ok > /nix/store/dgram-ok",
            &["nix/store/dgram-ok"],
        );
        allow_svc
            .do_build(ok_req)
            .await
            .expect("AllowInProcessPassiveDatagram must pass for in-process dgram");
        let allow_facts = allow_svc.last_enforcement_facts().unwrap();
        assert!(allow_facts.enforced_memory);
        assert!(allow_facts.enforced_pids);
        assert!(allow_facts.helper_checks.iter().any(|outcome| matches!(
            outcome,
            crate::buildservice::SandboxHelperScopeOutcome::InProcessPassiveDatagram { socket_kind }
                if socket_kind == "rust-waits"
        )));

        // 2. In-process UnixDatagram + RequireContained -> refused before payload start.
        let mut strict_policy = allow_policy.clone();
        strict_policy.resources.helper_scope_policy = crate::buildservice::SandboxHelperScopePolicy::RequireContained;
        let strict_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(strict_policy)
            .unwrap()
            .with_rust_wait_socket(in_proc_wait_sock)
            .unwrap();
        let strict_err = strict_svc
            .do_build(shell_build_request("/bin/busybox echo nope > /nix/store/out", &["nix/store/out"]))
            .await
            .unwrap_err();
        let strict_classified = strict_err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .unwrap();
        assert_eq!(strict_classified.class, crate::buildservice::SandboxFailureClass::ResourceEnforcementUnavailable);
        let strict_facts = strict_svc.last_enforcement_facts().unwrap();
        assert!(!strict_facts.enforced_memory);
        assert!(!strict_facts.enforced_pids);
        assert!(strict_facts.unavailable_reasons.iter().any(|r| matches!(
            r,
            crate::buildservice::SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { socket_kind, .. }
                if socket_kind == "rust-waits"
        )));

        // 3. External-process UnixDatagram + AllowInProcessPassiveDatagram -> refused.
        let ext_wait_sock = sock_dir.path().join("external-waits.sock");
        let exe = std::env::current_exe().unwrap();
        let mut ext_child = std::process::Command::new(exe)
            .env_remove("MANTLE_DELEGATED_CHILD")
            .env_remove("MANTLE_TEST_CGROUP_ROOT")
            .env("MANTLE_TEST_EXTERNAL_DGRAM_SOCKET", &ext_wait_sock)
            .args([
                "--exact",
                "buildservice::bwrap::tests::delegated_cgroup_dgram_helper_scope_and_pids_limit_controls",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut reader = BufReader::new(ext_child.stdout.take().unwrap());
        let mut line = String::new();
        let mut ready_seen = false;
        for _ in 0..32 {
            line.clear();
            let n = reader.read_line(&mut line).unwrap();
            assert!(n > 0, "external dgram helper exited before publishing READY");
            if line.trim() == "READY" {
                ready_seen = true;
                break;
            }
        }
        assert!(ready_seen, "external dgram helper did not publish READY within 32 stdout lines");

        let ext_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(allow_policy)
            .unwrap()
            .with_rust_wait_socket(ext_wait_sock)
            .unwrap();
        let ext_err = ext_svc
            .do_build(shell_build_request("/bin/busybox echo nope > /nix/store/out", &["nix/store/out"]))
            .await
            .unwrap_err();
        let ext_classified = ext_err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .unwrap();
        assert_eq!(ext_classified.class, crate::buildservice::SandboxFailureClass::ResourceEnforcementUnavailable);
        let ext_facts = ext_svc.last_enforcement_facts().unwrap();
        assert!(ext_facts.unavailable_reasons.iter().any(|r| matches!(
            r,
            crate::buildservice::SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { socket_kind, .. }
                if socket_kind == "rust-waits"
        )));
        drop(ext_child.stdin.take());
        assert!(ext_child.wait().unwrap().success());

        // 4. pids.max = 6 exhaustion -> pids_max_events > 0 and ResourceBudgetExceeded.
        let mut pids_policy = crate::buildservice::SandboxExecutionPolicy::standard();
        pids_policy.resources.cgroup_root = Some(cg_root.clone());
        pids_policy.resources.memory_mode = crate::buildservice::SandboxEnforcementMode::Required;
        pids_policy.resources.memory_limit_bytes = Some(64 * 1024 * 1024);
        pids_policy.resources.swap_policy = crate::buildservice::SandboxSwapPolicy::Zero;
        pids_policy.resources.pids_mode = crate::buildservice::SandboxEnforcementMode::Required;
        pids_policy.resources.pids_limit_count = Some(6);
        let pids_svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(pids_policy.clone())
            .unwrap();
        let pids_req = shell_build_request(
            "/bin/busybox sh -c '/bin/busybox sleep 1 & /bin/busybox sleep 1 & /bin/busybox sleep 1 & /bin/busybox sleep 1 & /bin/busybox sleep 1 & ( : ) || exit 1' && /bin/busybox echo unexpected > /nix/store/pids-out",
            &["nix/store/pids-out"],
        );
        let pids_err = pids_svc.do_build(pids_req).await.unwrap_err();
        let pids_classified = pids_err
            .get_ref()
            .and_then(|e| e.downcast_ref::<crate::buildservice::ClassifiedBuildServiceError>())
            .expect("pids exhaustion must produce ClassifiedBuildServiceError");
        assert_eq!(pids_classified.class, crate::buildservice::SandboxFailureClass::ResourceBudgetExceeded);
        let pids_facts = pids_svc.last_enforcement_facts().unwrap();
        assert!(pids_facts.pids_max_events > 0, "expected pids_max_events > 0, got {pids_facts:?}");
        assert_eq!(pids_facts.effective_pids_max, crate::buildservice::SandboxEffectiveLimit::Count(6));

        // 5. command.spawn() failure with cgroup enabled resets enforced_memory/enforced_pids to false and
        //    leaves zero attempt directories.
        struct BadSpawnFactory;
        impl crate::bwrap::BwrapCommandFactory for BadSpawnFactory {
            fn command(
                &self,
                _invocation: crate::bwrap::BwrapInvocation<'_>,
            ) -> std::io::Result<tokio::process::Command> {
                Ok(tokio::process::Command::new("/nonexistent/mantle-missing-bwrap"))
            }
        }
        let spawn_fail_svc =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_execution_policy(pids_policy.clone())
                .unwrap()
                .with_command_factory(Arc::new(BadSpawnFactory));
        let spawn_err = spawn_fail_svc
            .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out", &["nix/store/out"]))
            .await
            .unwrap_err();
        assert_eq!(spawn_err.kind(), ErrorKind::NotFound);
        let spawn_facts = spawn_fail_svc.last_enforcement_facts().unwrap();
        assert!(!spawn_facts.enforced_memory, "spawn failure must not claim memory enforcement");
        assert!(!spawn_facts.enforced_pids, "spawn failure must not claim pids enforcement");

        // 6. PanicAfterPid1 with delegated cgroup cleans up AttemptCgroup directory cleanly.
        let mut panic_svc =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_execution_policy(pids_policy)
                .unwrap();
        panic_svc.watch_fault = Some(WatchFault::PanicAfterPid1);
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let panic_err = panic_svc
            .do_build_cancellable(
                shell_build_request("/bin/busybox echo ok > /nix/store/out", &["nix/store/out"]),
                cancel_rx,
            )
            .await
            .unwrap_err();
        assert!(panic_err.to_string().contains("watch supervisor failed"));
        let panic_facts = panic_svc.last_enforcement_facts().unwrap();
        assert!(!panic_facts.enforced_memory);
        assert!(!panic_facts.enforced_pids);

        let leftover_attempt_dirs: Vec<_> = std::fs::read_dir(&cg_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("mantle-sbx-"))
            .collect();
        assert!(
            leftover_attempt_dirs.is_empty(),
            "delegated cgroup root leaked attempt directories: {leftover_attempt_dirs:?}"
        );
    }

    #[tokio::test]
    async fn delegated_cgroup_live_cpu_diagnostics_cpu_sleep_parallel_and_cancel() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let Some(cg_root) = resolve_or_spawn_delegated_cgroup_root(
            "buildservice::bwrap::tests::delegated_cgroup_live_cpu_diagnostics_cpu_sleep_parallel_and_cancel",
        ) else {
            return;
        };

        let root = tempfile::tempdir().unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.resources.cgroup_root = Some(cg_root.clone());
        // Diagnostics-only mode: memory_mode and pids_mode remain Disabled (no subtree_control needed).
        assert!(!policy.resources.requests_cgroup());

        let diag_cfg = crate::buildservice::SandboxLiveCpuDiagnosticsConfig {
            sample_cadence_ms: 50,
            allocated_cores: 2,
            generation: 1,
            cgroup_root: None,
        };
        let svc = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_execution_policy(policy)
            .unwrap()
            .with_live_cpu_diagnostics(diag_cfg, tx)
            .unwrap();

        // 1. CPU-bound workload: busy loop for ~200ms -> positive cpu delta.
        let cpu_req = shell_build_request(
            "i=0; while [ \"$i\" -lt 80000 ]; do i=$((i+1)); done; /bin/busybox echo cpu-ok > /nix/store/cpu-out",
            &["nix/store/cpu-out"],
        );
        svc.do_build(cpu_req).await.expect("cpu-bound diagnostics build must succeed");
        let cpu_facts = svc.last_live_cpu_facts().unwrap();
        assert!(
            matches!(cpu_facts.scope, crate::buildservice::SandboxCpuScopeAvailability::OwnedAttemptCgroupV2 { .. }),
            "expected owned cgroup scope in diagnostics-only mode, got {:?}",
            cpu_facts.scope
        );
        assert_eq!(cpu_facts.retired_reason, Some(crate::buildservice::SandboxCpuRetirementReason::Completed));
        assert!(cpu_facts.sample_count >= 2, "expected >=2 cpu.stat samples, got {}", cpu_facts.sample_count);
        assert!(
            cpu_facts.latest_cpu_usec.unwrap_or(0) > cpu_facts.initial_cpu_usec.unwrap_or(0),
            "cpu-bound workload must show positive usage_usec delta: {cpu_facts:?}"
        );

        // 2. Sleeping workload: sleep 250ms -> low cpu delta over >=200ms wall interval.
        while rx.try_recv().is_ok() {}
        let sleep_req =
            shell_build_request("/bin/busybox usleep 250000; /bin/busybox echo sleep-ok > /nix/store/sleep-out", &[
                "nix/store/sleep-out",
            ]);
        svc.do_build(sleep_req).await.expect("sleeping diagnostics build must succeed");
        let sleep_facts = svc.last_live_cpu_facts().unwrap();
        assert_eq!(sleep_facts.retired_reason, Some(crate::buildservice::SandboxCpuRetirementReason::Completed));
        let mut sleep_events = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            sleep_events.push(ev);
        }
        assert!(
            sleep_events
                .iter()
                .any(|ev| matches!(ev.kind, crate::buildservice::SandboxCpuSampleKind::Sample { .. }))
        );
        assert!(sleep_events.iter().any(|ev| matches!(ev.kind, crate::buildservice::SandboxCpuSampleKind::Retired {
            reason: crate::buildservice::SandboxCpuRetirementReason::Completed
        })));

        // 3. Parallel descendants: 2 concurrent busy loops inside cgroup -> tree cpu delta > wall dt
        //    (>1_000 milli-cores).
        while rx.try_recv().is_ok() {}
        let par_req = shell_build_request(
            "/bin/busybox sh -c 'i=0; while [ \"$i\" -lt 250000 ]; do i=$((i+1)); done' & \
             /bin/busybox sh -c 'i=0; while [ \"$i\" -lt 250000 ]; do i=$((i+1)); done' & \
             wait; /bin/busybox echo par-ok > /nix/store/par-out",
            &["nix/store/par-out"],
        );
        svc.do_build(par_req).await.expect("parallel diagnostics build must succeed");
        let mut samples: Vec<(u64, u64)> = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            if let crate::buildservice::SandboxCpuSampleKind::Sample { cumulative_cpu_usec } = ev.kind {
                samples.push((ev.observed_at_usec, cumulative_cpu_usec));
            }
        }
        assert!(samples.len() >= 2, "expected >=2 samples for parallel descendants, got {}", samples.len());
        let mut peak_milli_cores: u128 = 0;
        for win in samples.windows(2) {
            let (t0, c0) = win[0];
            let (t1, c1) = win[1];
            if t1 > t0 {
                let rate = (u128::from(c1.saturating_sub(c0)) * 1_000) / u128::from(t1.saturating_sub(t0));
                if rate > peak_milli_cores {
                    peak_milli_cores = rate;
                }
            }
        }
        let (t_first, c_first) = samples[0];
        let (t_last, c_last) = *samples.last().unwrap();
        let avg_milli_cores = if t_last > t_first {
            (u128::from(c_last.saturating_sub(c_first)) * 1_000) / u128::from(t_last.saturating_sub(t_first))
        } else {
            0
        };
        assert!(
            peak_milli_cores > 1_000,
            "expected parallel descendants peak > 1000 milli-cores, got peak={peak_milli_cores} avg={avg_milli_cores} samples={samples:?}"
        );

        // 4. Cancellation retires sampler synchronously before WatchLease drop and leaks zero post-cancel
        //    samples.
        while rx.try_recv().is_ok() {}
        let markers = tempfile::tempdir().unwrap();
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let cancel_svc = Arc::new(svc);
        let cancel_clone = Arc::clone(&cancel_svc);
        let cancel_req = watch_build_request(markers.path());
        let handle = tokio::spawn(async move { cancel_clone.do_build_cancellable(cancel_req, cancel_rx).await });
        await_watch_start(markers.path()).await;
        cancel_tx.send(true).unwrap();
        let cancel_err = handle.await.unwrap().unwrap_err();
        assert_eq!(cancel_err.kind(), ErrorKind::Interrupted);
        let cancel_facts = cancel_svc.last_live_cpu_facts().unwrap();
        assert_eq!(cancel_facts.retired_reason, Some(crate::buildservice::SandboxCpuRetirementReason::Cancelled));
        let mut saw_retired = false;
        while let Ok(ev) = rx.try_recv() {
            if ev.attempt_id == cancel_facts.attempt_id {
                assert!(!saw_retired, "no sample may arrive after Retired event for cancelled attempt: {ev:?}");
                if matches!(ev.kind, crate::buildservice::SandboxCpuSampleKind::Retired {
                    reason: crate::buildservice::SandboxCpuRetirementReason::Cancelled
                }) {
                    saw_retired = true;
                }
            }
        }
        assert!(saw_retired, "expected synchronous Retired(Cancelled) event before return");

        let leftover_attempt_dirs: Vec<_> = std::fs::read_dir(&cg_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("mantle-sbx-"))
            .collect();
        assert!(
            leftover_attempt_dirs.is_empty(),
            "delegated live cpu cgroup root leaked attempt directories: {leftover_attempt_dirs:?}"
        );
    }

    #[tokio::test]
    async fn live_cpu_diagnostics_unavailable_fallback_and_resilience_never_block_builds() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let root = tempfile::tempdir().unwrap();

        // 1. Missing cgroup_root -> honest Unavailable, build succeeds normally.
        let (tx1, mut rx1) = tokio::sync::mpsc::channel(16);
        let svc_unavail =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_live_cpu_diagnostics(crate::buildservice::SandboxLiveCpuDiagnosticsConfig::standard(2), tx1)
                .unwrap();
        svc_unavail
            .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out1", &["nix/store/out1"]))
            .await
            .expect("build must succeed when live cpu cgroup_root is unconfigured");
        let facts1 = svc_unavail.last_live_cpu_facts().unwrap();
        assert!(matches!(facts1.scope, crate::buildservice::SandboxCpuScopeAvailability::Unavailable {
            reason: crate::buildservice::SandboxLiveCpuUnavailableReason::CgroupRootUnconfigured
        }));
        assert_eq!(facts1.retired_reason, Some(crate::buildservice::SandboxCpuRetirementReason::Completed));
        let mut events1 = Vec::new();
        while let Ok(ev) = rx1.try_recv() {
            events1.push(ev);
        }
        assert!(events1.iter().any(|ev| matches!(ev.kind, crate::buildservice::SandboxCpuSampleKind::Started {
            scope: crate::buildservice::SandboxCpuScopeAvailability::Unavailable { .. }
        })));

        // 2. Full bounded channel (capacity 1) never blocks build and counts dropped samples.
        let (tx_full, _rx_full_kept_unread) = tokio::sync::mpsc::channel(1);
        let svc_full = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_live_cpu_diagnostics(crate::buildservice::SandboxLiveCpuDiagnosticsConfig::standard(1), tx_full)
            .unwrap();
        svc_full
            .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out2", &["nix/store/out2"]))
            .await
            .expect("full bounded channel must never block build");
        let facts_full = svc_full.last_live_cpu_facts().unwrap();
        assert!(facts_full.dropped_samples_count >= 1);

        // 3. Dropped consumer mid-flight (rx dropped immediately) never blocks or fails build.
        let (tx_closed, rx_closed) = tokio::sync::mpsc::channel(4);
        drop(rx_closed);
        let svc_closed = BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
            .with_live_cpu_diagnostics(crate::buildservice::SandboxLiveCpuDiagnosticsConfig::standard(1), tx_closed)
            .unwrap();
        svc_closed
            .do_build(shell_build_request("/bin/busybox echo ok > /nix/store/out3", &["nix/store/out3"]))
            .await
            .expect("dropped diagnostic consumer must never fail or block build");
        let facts_closed = svc_closed.last_live_cpu_facts().unwrap();
        assert_eq!(facts_closed.retired_reason, Some(crate::buildservice::SandboxCpuRetirementReason::Completed));
        assert!(facts_closed.dropped_samples_count >= 1);
    }

    #[tokio::test]
    async fn delegated_cgroup_live_cpu_observer_overhead_fixture() {
        if !require_real_bwrap_and_shell() {
            return;
        }
        let Some(cg_root) = resolve_or_spawn_delegated_cgroup_root(
            "buildservice::bwrap::tests::delegated_cgroup_live_cpu_observer_overhead_fixture",
        ) else {
            return;
        };

        const ITERATIONS: usize = 8;
        const MICRO_ITERS: usize = 1_000;
        let root = tempfile::tempdir().unwrap();
        let mut policy = crate::buildservice::SandboxExecutionPolicy::standard();
        policy.resources.cgroup_root = Some(cg_root.clone());

        let baseline_svc =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_execution_policy(policy.clone())
                .unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let observed_svc =
            BubblewrapBuildService::new(root.path().to_path_buf(), MemoryBlobService::default(), tmp_ds())
                .with_execution_policy(policy)
                .unwrap()
                .with_live_cpu_diagnostics(crate::buildservice::SandboxLiveCpuDiagnosticsConfig::standard(2), tx)
                .unwrap();

        // Micro-benchmark: 1,000 cpu.stat openat+read+parse + try_send iterations on an owned cgroup.
        let micro_cg = crate::bwrap::cgroup::prepare_attempt_diagnostics(Some(&cg_root)).unwrap();
        let (micro_tx, mut micro_rx) = tokio::sync::mpsc::channel(256);
        let micro_start = std::time::Instant::now();
        for seq in 0..MICRO_ITERS {
            let usage = micro_cg.read_cpu_usage_usec().unwrap();
            let _ = micro_tx.try_send(crate::buildservice::SandboxCpuSampleEvent {
                attempt_id: 1,
                generation: 1,
                attempt_label: "micro".to_string(),
                allocated_cores: 2,
                observed_at_usec: u64::try_from(seq).unwrap_or(0),
                kind: crate::buildservice::SandboxCpuSampleKind::Sample {
                    cumulative_cpu_usec: usage,
                },
            });
            let _ = micro_rx.try_recv();
        }
        let micro_total_us = micro_start.elapsed().as_micros();
        let micro_per_sample_ns = (micro_total_us.saturating_mul(1_000)) / u128::try_from(MICRO_ITERS).unwrap();
        drop(micro_cg);

        let script = "/bin/busybox usleep 600000; /bin/busybox echo ok > /nix/store/bench-out";

        // Warm-up 1 build each.
        baseline_svc.do_build(shell_build_request(script, &["nix/store/bench-out"])).await.unwrap();
        observed_svc.do_build(shell_build_request(script, &["nix/store/bench-out"])).await.unwrap();
        while rx.try_recv().is_ok() {}

        let mut baseline_us: Vec<u128> = Vec::with_capacity(ITERATIONS);
        let mut observed_us: Vec<u128> = Vec::with_capacity(ITERATIONS);
        let mut total_samples: u64 = 0;
        for _ in 0..ITERATIONS {
            let t0 = std::time::Instant::now();
            baseline_svc.do_build(shell_build_request(script, &["nix/store/bench-out"])).await.unwrap();
            baseline_us.push(t0.elapsed().as_micros());

            let t1 = std::time::Instant::now();
            observed_svc.do_build(shell_build_request(script, &["nix/store/bench-out"])).await.unwrap();
            observed_us.push(t1.elapsed().as_micros());
            if let Some(facts) = observed_svc.last_live_cpu_facts() {
                total_samples = total_samples.saturating_add(facts.sample_count);
            }
            while rx.try_recv().is_ok() {}
        }

        baseline_us.sort_unstable();
        observed_us.sort_unstable();
        let base_med = baseline_us[ITERATIONS / 2];
        let obs_med = observed_us[ITERATIONS / 2];
        let base_mean = baseline_us.iter().sum::<u128>() / u128::try_from(ITERATIONS).unwrap();
        let obs_mean = observed_us.iter().sum::<u128>() / u128::try_from(ITERATIONS).unwrap();
        let delta_med_us = obs_med.saturating_sub(base_med);
        let overhead_bps = (delta_med_us.saturating_mul(10_000)).checked_div(base_med).unwrap_or(0);
        eprintln!(
            "OVERHEAD_FIXTURE iterations={ITERATIONS} cadence_ms=500 micro_per_sample_ns={micro_per_sample_ns} baseline_median_us={base_med} observed_median_us={obs_med} baseline_mean_us={base_mean} observed_mean_us={obs_mean} total_samples={total_samples} overhead_bps={overhead_bps}"
        );
        assert!(total_samples >= u64::try_from(ITERATIONS).unwrap());
    }
}
