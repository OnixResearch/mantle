use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::watch;

use crate::buildservice::SandboxExecutionPolicy;
use crate::buildservice::SandboxFailureClass;
use crate::buildservice::SandboxResourceEnforcementFacts;
use crate::buildservice::classified_io_error;
use crate::sandbox::InputsProvider;
use crate::sandbox::SandboxSpec;
pub(crate) mod capture;
#[cfg(target_os = "linux")]
pub(crate) mod cgroup;
#[cfg(target_os = "linux")]
pub(crate) mod watch_cancel;

const BWRAP_PROGRAM: &str = "bwrap";
const BWRAP_PATH_ENV: &str = "SNIX_BUILD_BWRAP";

/// Convert a sandbox environment value to an argument without assuming UTF-8.
///
/// Unix preserves the declared bytes; platforms whose process arguments are
/// Unicode require valid UTF-8. NUL is never admissible in either case.
fn sandbox_env_value(value: &[u8]) -> std::io::Result<OsString> {
    if value.len() > 16 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "sandbox environment value exceeds the 16 MiB bound",
        ));
    }
    if value.contains(&0) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "sandbox environment value contains NUL"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Ok(OsStr::from_bytes(value).to_owned())
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(value).map(OsString::from).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "sandbox environment value is not UTF-8 on this platform",
            )
        })
    }
}

/// All paths are host paths except `inputs_dir`, which is the relative guest
/// input directory supplied by [`InputsProvider::inputs_dir`].
pub struct BwrapInvocation<'a> {
    pub program: &'a OsStr,
    pub args: &'a [OsString],
    pub host_workdir: &'a Path,
    pub host_inputs_dir: &'a Path,
    pub inputs_dir: &'a Path,
}

/// Optionally wrap the sandbox launcher without changing build identity.
///
/// The factory must return an unspawned command containing the complete
/// invocation (including its arguments). The executor then applies its stdin,
/// umask and watch supervision policy; it does not append these arguments again.
/// Input mounts remain alive throughout this callback and command execution.
pub trait BwrapCommandFactory: Send + Sync {
    fn command(&self, invocation: BwrapInvocation<'_>) -> std::io::Result<Command>;
}

const COMMON_BWRAP_ARGS: &[&str] = &[
    "--unshare-uts",
    "--hostname",
    "localhost",
    "--unshare-ipc",
    "--unshare-pid",
    // NOTE: --die-with-parent remains omitted because PR_SET_PDEATHSIG(SIGKILL)
    // fires when the spawning tokio worker thread exits, killing long-running
    // sandbox builds. Keep --as-pid-1 omitted too: bubblewrap's PID-1 reaper
    // must own command teardown, especially for short-lived builders inside a
    // nested user namespace.
    "--unshare-user",
    "--unshare-cgroup-try",
    // Prevent sandbox from gaining new privileges via setuid/setgid binaries
    // or other capability escalation.
    "--new-session",
    "--uid",
    "1000",
    "--gid",
    "100",
    "--clearenv",
    "--tmpfs",
    "/",
    "--dev",
    "/dev",
    "--tmpfs",
    "/dev/shm",
    "--proc",
    "/proc",
    "--tmpfs",
    "/tmp",
];

const RANDOM_DEVICE_MASK_ARGS: &[&str] = &[
    "--ro-bind",
    "/dev/null",
    "/dev/random",
    "--ro-bind",
    "/dev/null",
    "/dev/urandom",
];

const PROC_METADATA_MASK_ARGS: &[&str] = &[
    "--ro-bind-try",
    "/dev/null",
    "/proc/cpuinfo",
    "--ro-bind-try",
    "/dev/null",
    "/proc/meminfo",
    "--ro-bind-try",
    "/dev/null",
    "/proc/stat",
    "--ro-bind-try",
    "/dev/null",
    "/proc/loadavg",
    "--ro-bind-try",
    "/dev/null",
    "/proc/uptime",
    "--ro-bind-try",
    "/dev/null",
    "/proc/version",
];

const ETC_PASSWD: &[u8] = b"
root:x:0:0:Nix build user:/build:/noshell
nixbld:x:1000:100:Nix build user:/build:/noshell
nobody:x:65534:65534:Nobody:/:/noshell
";

const ETC_GROUP: &[u8] = b"
root:x:0:
nixbld:!:100:
nogroup:x:65534:
";

const ETC_HOSTS: &[u8] = b"
127.0.0.1 localhost
::1 localhost
";

const ETC_NSSWITCH: &[u8] = b"
hosts: files dns
services: files
";

const ETC_SERVICES: &[u8] = b"
tcpmux          1/tcp
echo            7/tcp
echo            7/udp
discard         9/tcp
discard         9/udp
systat          11/tcp
daytime         13/tcp
daytime         13/udp
qotd            17/tcp
chargen         19/tcp
chargen         19/udp
ftp-data        20/tcp
ftp             21/tcp
ssh             22/tcp
telnet          23/tcp
smtp            25/tcp
time            37/tcp
time            37/udp
nameserver      42/tcp
nicname         43/tcp
domain          53/tcp
domain          53/udp
bootps          67/udp
bootpc          68/udp
tftp            69/udp
gopher          70/tcp
http            80/tcp
kerberos        88/tcp
kerberos        88/udp
pop3            110/tcp
ident           113/tcp
sftp            115/tcp
nntp            119/tcp
ntp             123/udp
imap            143/tcp
snmp            161/udp
snmp-trap       162/udp
bgp             179/tcp
irc             194/tcp
ldap            389/tcp
https           443/tcp
smtps           465/tcp
submission      587/tcp
ldaps           636/tcp
imaps           993/tcp
pop3s           995/tcp
";

const SANDBOX_UMASK_BITS: u32 = 0o022;
const _: () = assert!(SANDBOX_UMASK_BITS <= 0o777);

/// Bubblewrap based sandbox executor.
///
/// It executes the sandbox command in separate uts, ipc, pid and user namespaces,
/// always runs as uid=1000(nixbld) and gid=100(nixbld) inside the namespace. Provides sane
/// defaults for various `/etc` files.
///
/// Network is optionally disabled with a separate network namespace based on the value of
/// [SandboxSpec::allow_network].
///
/// The root filesystem is tmpfs, has /dev and /proc.
///
/// The rest of the filesystem is based on the [SandboxSpec::scratches],
/// [SandboxSpec::additional_files] and [SandboxSpec::inputs_provider].
///
/// # Scratches
///
/// A list of read-write directories available inside the sandbox, these directories are also left
/// available on the host after the sandbox has finished.
///
/// # Additional files
///
/// A list of read-write files whose path currently *must* resolve into one of the Scratches.
///
/// # Build Inputs([SandboxSpec::inputs_provider])
///
/// A read-only directory that contains any files required by the sandboxed command, e.g
/// `/nix/store`.
/// Before the sandbox starts, the [SandboxSpec::inputs_provider] will have a chance to populate
/// this directory and clean up after the sandbox is stopped.
///
/// **Note**: If the build inputs directory overlaps with any of the scratches, an overlayfs mount
/// will be created for that scratch so it remains writable, i.e. the sandboxed command can create
/// new files/directories.
pub struct Bwrap {
    host_workdir: PathBuf,
    args: Vec<OsString>,
    inputs_provider: InputsProvider,
    command_factory: Option<Arc<dyn BwrapCommandFactory>>,
    execution_policy: SandboxExecutionPolicy,
    shutdown_signal: Option<watch::Receiver<bool>>,
    compile_socket: Option<PathBuf>,
    rust_wait_socket: Option<PathBuf>,
    provide_shell: Option<PathBuf>,
    has_bind_scratches: bool,
    enforcement_facts_sink: Option<Arc<Mutex<Option<SandboxResourceEnforcementFacts>>>>,
    #[cfg(test)]
    watch_fault: Option<watch_cancel::WatchFault>,
}

/// The result of running the sandbox.
pub struct SandboxOutcome {
    output: Output,
    scratch_dir: PathBuf,
}

impl SandboxOutcome {
    /// Status code, stderr, stdout, etc.
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// Allows finding outputs produced by the sandboxed command.
    ///
    /// The command must write into one of the scratches.
    pub fn find_path(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        let path = self.scratch_dir.join(path);
        // Exists follows symlinks so may return false incorrectly, as nix builds are apparently
        // allowed to produce broken symlinks as their $out...
        // i.e. `runCommand "test" {} "ln -s IdontExist $out"` is a valid nix build.
        //
        // Additionally, SandboxOutcome values are handed out by builds **after** unmounting the
        // fuse store, which means that even valid symlinks can be "broken" during ingestion.
        if path.is_symlink() || path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

#[cfg(unix)]
fn set_sandbox_umask(command: &mut Command) {
    // SAFETY: `pre_exec` runs in the forked child immediately before `execve`.
    // The closure performs one libc syscall (`umask`) and does not allocate or
    // touch shared Rust state.
    unsafe {
        command.pre_exec(|| {
            nix::libc::umask(SANDBOX_UMASK_BITS as nix::libc::mode_t);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn set_sandbox_umask(_command: &mut Command) {}

fn choose_bwrap_program(env_value: Option<OsString>) -> OsString {
    match env_value {
        Some(value) if !value.is_empty() => value,
        _ => OsString::from(BWRAP_PROGRAM),
    }
}

pub(crate) fn bwrap_program() -> OsString {
    choose_bwrap_program(std::env::var_os(BWRAP_PATH_ENV))
}

fn annotate_bwrap_spawn_error(program: &OsString, error: std::io::Error) -> std::io::Error {
    if error.kind() != ErrorKind::NotFound {
        return error;
    }

    std::io::Error::new(
        ErrorKind::NotFound,
        format!(
            "bubblewrap executable '{}' not found; put '{}' on PATH or set {} to an executable bwrap path: {error}",
            program.to_string_lossy(),
            BWRAP_PROGRAM,
            BWRAP_PATH_ENV
        ),
    )
}

fn append_prevalidated_mount_args<'a>(
    args: &mut Vec<OsString>,
    mounts: impl IntoIterator<Item = &'a crate::sandbox::SandboxMount>,
) -> std::io::Result<()> {
    for mount in mounts {
        if !mount.host_path.is_absolute() || !mount.guest_path.is_absolute() {
            return Err(std::io::Error::other("sandbox mount paths must be absolute"));
        }
        if mount
            .guest_path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        {
            return Err(std::io::Error::other("sandbox guest mount path must be clean"));
        }
        // Socket bind targets must not first be created as directories. Keep
        // the ordinary workspace-directory mount behavior byte-for-byte intact.
        if mount.guest_path == Path::new("/run/mantle/compile-cache.sock")
            || mount.guest_path == Path::new("/run/mantle/rust-waits.sock")
        {
            use std::os::unix::fs::FileTypeExt as _;
            if !mount.read_only || !fs::symlink_metadata(&mount.host_path)?.file_type().is_socket() {
                return Err(std::io::Error::other("compile runtime mount requires one read-only Unix socket"));
            }
            args.extend([
                "--dir".into(),
                "/run/mantle".into(),
                "--ro-bind".into(),
                mount.host_path.clone().into(),
                mount.guest_path.clone().into(),
            ]);
            continue;
        }
        let bind_flag = if mount.read_only { "--ro-bind" } else { "--bind" };
        args.extend([
            "--dir".into(),
            mount.guest_path.clone().into(),
            bind_flag.into(),
            mount.host_path.clone().into(),
            mount.guest_path.clone().into(),
        ]);
    }
    Ok(())
}

impl Bwrap {
    pub(crate) fn with_command_factory(mut self, command_factory: Arc<dyn BwrapCommandFactory>) -> Self {
        self.command_factory = Some(command_factory);
        self
    }

    pub(crate) fn with_execution_policy(mut self, policy: SandboxExecutionPolicy) -> Self {
        self.execution_policy = policy;
        self
    }

    pub(crate) fn with_shutdown_signal(mut self, signal: Option<watch::Receiver<bool>>) -> Self {
        self.shutdown_signal = signal;
        self
    }

    pub(crate) fn with_helper_sockets(
        mut self,
        compile_socket: Option<PathBuf>,
        rust_wait_socket: Option<PathBuf>,
    ) -> Self {
        self.compile_socket = compile_socket;
        self.rust_wait_socket = rust_wait_socket;
        self
    }

    pub(crate) fn with_enforcement_facts_sink(
        mut self,
        sink: Option<Arc<Mutex<Option<SandboxResourceEnforcementFacts>>>>,
    ) -> Self {
        self.enforcement_facts_sink = sink;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_test_watch_fault(mut self, fault: Option<watch_cancel::WatchFault>) -> Self {
        self.watch_fault = fault;
        self
    }

    pub async fn run(self) -> std::io::Result<SandboxOutcome> {
        self.run_inner(None, None).await
    }

    /// Run a supervised sandbox (one-shot or watch-cancellable) until its
    /// dedicated pidfd + gate supervisor confirms completion, deadline expiry,
    /// graceful shutdown, or cancellation. Dropping this future requests
    /// teardown; it cannot drop the owned PID1 gate, cgroup, or child process.
    pub(crate) async fn run_supervised(
        self,
        cancellation: Option<watch::Receiver<bool>>,
        lease: watch_cancel::WatchLease,
    ) -> std::io::Result<SandboxOutcome> {
        self.run_inner(cancellation, Some(lease)).await
    }

    async fn run_inner(
        mut self,
        cancellation: Option<watch::Receiver<bool>>,
        lease: Option<watch_cancel::WatchLease>,
    ) -> std::io::Result<SandboxOutcome> {
        if cancellation.as_ref().is_some_and(|receiver| *receiver.borrow() || receiver.has_changed().is_err()) {
            return Err(std::io::Error::new(ErrorKind::Interrupted, "watch build cancelled before sandbox start"));
        }
        if self
            .shutdown_signal
            .as_ref()
            .is_some_and(|receiver| *receiver.borrow() || receiver.has_changed().is_err())
        {
            return Err(std::io::Error::new(ErrorKind::Interrupted, "sandbox shutdown requested before start"));
        }
        self.execution_policy.validate()?;

        let program = bwrap_program();
        #[cfg(test)]
        let pidfd_injected_unavailable = matches!(self.watch_fault, Some(watch_cancel::WatchFault::PidfdUnavailable));
        #[cfg(not(test))]
        let pidfd_injected_unavailable = false;

        let sink = self.enforcement_facts_sink.take();
        if self.execution_policy.resources.requests_any_enforcement() {
            let prepared = cgroup::prepare_attempt_enforcement(cgroup::AttemptEnforcementInput {
                policy: &self.execution_policy.resources,
                bwrap_program: Path::new(&program),
                provide_shell: self.provide_shell.as_deref(),
                has_bind_scratches: self.has_bind_scratches,
                compile_socket: self.compile_socket.as_deref(),
                rust_wait_socket: self.rust_wait_socket.as_deref(),
                pidfd_injected_unavailable,
            });
            match prepared {
                Ok(prep) => {
                    let had_cgroup = prep.cgroup.is_some();
                    let facts_arc = Arc::new(Mutex::new(prep.facts));
                    let result = self
                        .run_with_prepared_cgroup(cancellation, lease, &program, prep.cgroup, Arc::clone(&facts_arc))
                        .await;
                    let mut final_facts = facts_arc.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
                    if result.is_err()
                        && had_cgroup
                        && !final_facts.populated_observed
                        && final_facts.observed_member_pids.is_empty()
                    {
                        final_facts.enforced_memory = false;
                        final_facts.enforced_pids = false;
                    }
                    if let Some(sink) = sink {
                        *sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(final_facts);
                    }
                    return result;
                }
                Err(boxed) => {
                    let (reason, facts) = *boxed;
                    if let Some(sink) = sink {
                        *sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(facts);
                    }
                    return Err(classified_io_error(
                        ErrorKind::Other,
                        SandboxFailureClass::ResourceEnforcementUnavailable,
                        format!("local sandbox resource enforcement unavailable: {reason:?}"),
                    ));
                }
            }
        }

        if pidfd_injected_unavailable {
            return Err(std::io::Error::new(ErrorKind::Unsupported, "injected preflight pidfd unavailability"));
        }
        watch_cancel::require_pidfd_support()?;

        let facts_arc = Arc::new(Mutex::new(SandboxResourceEnforcementFacts::default()));
        let result = self.run_with_prepared_cgroup(cancellation, lease, &program, None, Arc::clone(&facts_arc)).await;
        if let Some(sink) = sink {
            let final_facts = facts_arc.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
            *sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(final_facts);
        }
        result
    }

    async fn run_with_prepared_cgroup(
        mut self,
        cancellation: Option<watch::Receiver<bool>>,
        lease: Option<watch_cancel::WatchLease>,
        program: &OsString,
        attempt_cgroup: Option<Arc<cgroup::AttemptCgroup>>,
        enforcement_facts: Arc<Mutex<SandboxResourceEnforcementFacts>>,
    ) -> std::io::Result<SandboxOutcome> {
        watch_cancel::require_pidfd_support()?;
        // Transfer InputsGuard ownership into WatchLease so FUSE/materialized
        // inputs stay mounted until the owned supervisor completes PID1 teardown.
        let inputs_dir = self.inputs_provider.inputs_dir().to_path_buf();
        let host_inputs_dir = self.host_workdir.join("host_inputs_dir");
        let inputs_guard = self.inputs_provider.into_guard(&host_inputs_dir)?;
        let lease = match lease {
            Some(mut existing) => {
                existing.attach_inputs_guard(inputs_guard);
                existing
            }
            None => watch_cancel::WatchLease::from_inputs_only(inputs_guard),
        };

        let gate = watch_cancel::WatchStartGate::new()?;
        self.args.splice(0..0, gate.block_arguments());
        #[cfg(test)]
        if matches!(self.watch_fault, Some(watch_cancel::WatchFault::ExitBeforePid1)) {
            self.args.insert(0, "--invalid-bwrap-watch-early-exit".into());
        }
        let mut command = match self.command_factory.as_ref() {
            Some(factory) => factory.command(BwrapInvocation {
                program,
                args: &self.args,
                host_workdir: &self.host_workdir,
                host_inputs_dir: &host_inputs_dir,
                inputs_dir: &inputs_dir,
            })?,
            None => {
                let mut command = Command::new(program);
                command.args(&self.args);
                command
            }
        };
        gate.configure(&mut command);
        let (pending_cgroup_spawn, cgroup_ack_reader) = match attempt_cgroup.as_ref() {
            Some(cg) => {
                let (pending, ack_reader) = cg.configure_pre_exec(&mut command)?;
                (Some(pending), Some(ack_reader))
            }
            None => (None, None),
        };
        command.stdin(Stdio::null());
        set_sandbox_umask(&mut command);

        let supervisor_config = watch_cancel::SupervisorRunConfig {
            wall_deadline: self.execution_policy.wall_deadline.as_duration(),
            silence_budget: self.execution_policy.silence_budget.as_duration(),
            shutdown_grace: Duration::from_millis(self.execution_policy.shutdown_grace_ms),
            shutdown_signal: self.shutdown_signal,
            cgroup: attempt_cgroup,
            cgroup_ack_reader,
            pending_cgroup_spawn,
            enforcement_facts,
        };
        let output = watch_cancel::run_child(
            command,
            program,
            cancellation,
            gate,
            lease,
            supervisor_config,
            #[cfg(test)]
            self.watch_fault,
        )
        .await?;

        Ok(SandboxOutcome {
            output,
            scratch_dir: self.host_workdir.join("scratches"),
        })
    }

    /// Constructor.
    pub fn initialize(spec: SandboxSpec) -> std::io::Result<Bwrap> {
        let scratch_dir = spec.host_workdir().join("scratches");
        fs::create_dir_all(&scratch_dir)?;
        let mut args: Vec<OsString> = COMMON_BWRAP_ARGS.iter().map(|s| s.into()).collect();
        if !spec.provide_random_devices() {
            args.extend(RANDOM_DEVICE_MASK_ARGS.iter().map(OsString::from));
        }
        if !spec.provide_proc_metadata() {
            args.extend(PROC_METADATA_MASK_ARGS.iter().map(OsString::from));
        }
        if !spec.allow_network() {
            args.push("--unshare-net".into());
        }
        for env in spec.env_vars() {
            if env.key.is_empty() || env.key.as_bytes().contains(&0) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "sandbox environment variable name is empty or contains NUL",
                ));
            }
            args.extend([
                "--setenv".into(),
                env.key.clone().into(),
                sandbox_env_value(&env.value)?,
            ]);
        }

        let host_inputs_dir = spec.host_workdir().join("host_inputs_dir");
        fs::create_dir_all(&host_inputs_dir)?;
        args.extend([
            "--ro-bind".into(),
            Path::new("/").join(&host_inputs_dir).into(),
            Path::new("/").join(spec.inputs_provider().inputs_dir()).into(),
        ]);
        for scratch in spec.scratches() {
            let scratch_path = scratch_dir.join(scratch);
            fs::create_dir_all(&scratch_path)?;
            if scratch == spec.inputs_provider().inputs_dir() {
                let overlay_workdir = spec.host_workdir().join("overlay_workdir");
                fs::create_dir_all(&overlay_workdir)?;
                args.extend([
                    "--overlay-src".into(),
                    OsString::from(&host_inputs_dir),
                    "--overlay".into(),
                    scratch_path.into(),
                    overlay_workdir.into(),
                    Path::new("/").join(spec.inputs_provider().inputs_dir()).into(),
                ]);
            } else {
                args.extend([
                    "--bind".into(),
                    scratch_path.into(),
                    Path::new("/").join(scratch).into(),
                ]);
            }
        }
        append_prevalidated_mount_args(&mut args, spec.mounts())?;
        args.extend(["--chdir".into(), Path::new("/").join(spec.sandbox_workdir()).into()]);

        if let Some(shell) = spec.provide_shell() {
            args.extend_from_slice(&["--ro-bind".into(), shell.into(), "/bin/sh".into()]);
            // Also mount as /bin/busybox so that busybox applets can be
            // invoked via `busybox <applet>` or via symlinks named after
            // the applet pointing to /bin/busybox.
            args.extend_from_slice(&["--ro-bind".into(), shell.into(), "/bin/busybox".into()]);
        }

        for file in spec.additional_files() {
            let mut found = false;
            for scratch in spec.scratches() {
                if file.path.starts_with(scratch) {
                    found = true;
                }
            }
            if !found {
                return Err(std::io::Error::other(format!(
                    "Additional file does not belong to any scratch: {:?}",
                    file.path
                )));
            }
            // TODO: prevent files from escaping the sandbox, i.e. don't allow additional files
            // of this form: build/../../hello.
            let file_path = scratch_dir.join(&file.path);
            fs::create_dir_all(file_path.parent().expect("parent"))?;
            fs::write(&file_path, &file.contents)?;
        }
        let etc = &spec.host_workdir().join("etc");
        fs::create_dir_all(etc)?;
        fs::write(etc.join("passwd"), ETC_PASSWD)?;
        fs::write(etc.join("group"), ETC_GROUP)?;
        fs::write(etc.join("hosts"), ETC_HOSTS)?;
        fs::write(etc.join("nsswitch.conf"), ETC_NSSWITCH)?;

        args.extend([
            "--ro-bind".into(),
            etc.join("passwd").into(),
            "/etc/passwd".into(),
            "--ro-bind".into(),
            etc.join("group").into(),
            "/etc/group".into(),
        ]);
        if spec.allow_network() {
            let Some(dns_config) = spec.dns_config() else {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "network-enabled sandbox requires an explicit validated DNS resolver configuration",
                ));
            };
            let rendered_resolv_conf = dns_config
                .render_resolv_conf()
                .map_err(|error| std::io::Error::new(ErrorKind::InvalidInput, error))?;
            fs::write(etc.join("resolv.conf"), rendered_resolv_conf.as_bytes())?;
            fs::write(etc.join("services"), ETC_SERVICES)?;
            args.extend([
                "--ro-bind".into(),
                etc.join("hosts").into(),
                "/etc/hosts".into(),
                "--ro-bind".into(),
                etc.join("resolv.conf").into(),
                "/etc/resolv.conf".into(),
                "--ro-bind".into(),
                etc.join("services").into(),
                "/etc/services".into(),
                "--ro-bind".into(),
                etc.join("nsswitch.conf").into(),
                "/etc/nsswitch.conf".into(),
            ]);
        } else {
            if spec.dns_config().is_some() {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "DNS resolver configuration is forbidden when sandbox network access is denied",
                ));
            }
            // Use predefined /etc/hosts like nix does.
            // Among other things it is required for libuv getaddrinfo() tests to pass.
            args.extend(["--ro-bind".into(), etc.join("hosts").into(), "/etc/hosts".into()]);
        }
        args.extend(spec.command().into_iter().map(|s| s.into()));

        let has_bind_scratches = spec.scratches().into_iter().next().is_some();
        let provide_shell = spec.provide_shell().map(Path::to_path_buf);

        Ok(Self {
            host_workdir: spec.host_workdir().into(),
            args,
            inputs_provider: spec.into(),
            command_factory: None,
            execution_policy: SandboxExecutionPolicy::standard(),
            shutdown_signal: None,
            compile_socket: None,
            rust_wait_socket: None,
            provide_shell,
            has_bind_scratches,
            enforcement_facts_sink: None,
            #[cfg(test)]
            watch_fault: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PROC_MASKS: &[&str] = &[
        "/proc/cpuinfo",
        "/proc/meminfo",
        "/proc/stat",
        "/proc/loadavg",
        "/proc/uptime",
        "/proc/version",
    ];

    fn minimal_spec(host_workdir: &Path, allow_network: bool) -> SandboxSpec {
        SandboxSpec::builder()
            .host_workdir(host_workdir)
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .allow_network(allow_network)
            .build()
    }

    fn arg_strings(bwrap: &Bwrap) -> Vec<String> {
        bwrap.args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
    }

    fn bind_source_for(args: &[String], dest: &str) -> Option<String> {
        args.windows(3)
            .find(|window| window[0] == "--ro-bind" && window[2] == dest)
            .map(|window| window[1].clone())
    }

    #[test]
    fn initialize_preserves_non_utf8_env_values() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .env_vars(vec![crate::buildservice::EnvVar {
                key: "MANTLE_BYTES".to_string(),
                value: bytes::Bytes::from_static(&[0xff, 0xfe, 0x80]),
            }])
            .build();
        let bwrap = Bwrap::initialize(spec).expect("non-UTF-8 env bytes must not panic");
        let index = bwrap.args.iter().position(|arg| arg == "--setenv").expect("setenv argument exists");
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            assert_eq!(bwrap.args[index + 2], OsStr::from_bytes(&[0xff, 0xfe, 0x80]));
        }
        #[cfg(not(unix))]
        assert_eq!(bwrap.args[index + 2], OsStr::new(""));
    }

    #[test]
    fn initialize_rejects_nul_env_value_before_spawn() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .env_vars(vec![crate::buildservice::EnvVar {
                key: "MANTLE_BYTES".to_string(),
                value: bytes::Bytes::from_static(b"a\0b"),
            }])
            .build();
        let error = match Bwrap::initialize(spec) {
            Ok(_) => panic!("NUL env value was accepted"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert!(error.to_string().contains("NUL"));
    }

    #[test]
    fn choose_bwrap_program_prefers_explicit_env_path() {
        let explicit = OsString::from("/tools/bwrap");
        let selected = choose_bwrap_program(Some(explicit.clone()));

        assert_eq!(selected, explicit);
    }

    #[test]
    fn choose_bwrap_program_falls_back_for_empty_or_missing_env() {
        assert_eq!(choose_bwrap_program(None), OsString::from(BWRAP_PROGRAM));
        assert_eq!(choose_bwrap_program(Some(OsString::new())), OsString::from(BWRAP_PROGRAM));
    }

    #[test]
    fn annotate_bwrap_spawn_error_mentions_env_for_missing_program() {
        let error = std::io::Error::new(ErrorKind::NotFound, "missing");
        let annotated = annotate_bwrap_spawn_error(&OsString::from("/missing/bwrap"), error);
        let rendered = annotated.to_string();

        assert_eq!(annotated.kind(), ErrorKind::NotFound);
        assert!(rendered.contains("/missing/bwrap"));
        assert!(rendered.contains(BWRAP_PATH_ENV));
    }

    #[test]
    fn annotate_bwrap_spawn_error_preserves_non_not_found_errors() {
        let error = std::io::Error::new(ErrorKind::PermissionDenied, "denied");
        let annotated = annotate_bwrap_spawn_error(&OsString::from("/locked/bwrap"), error);

        assert_eq!(annotated.kind(), ErrorKind::PermissionDenied);
        assert_eq!(annotated.to_string(), "denied");
    }

    #[test]
    fn common_bwrap_args_contains_hostname() {
        assert!(COMMON_BWRAP_ARGS.windows(2).any(|w| w == ["--hostname", "localhost"]));
    }

    #[test]
    fn common_bwrap_args_install_pid_namespace_reaper() {
        assert!(COMMON_BWRAP_ARGS.contains(&"--unshare-pid"));
        assert!(!COMMON_BWRAP_ARGS.contains(&"--as-pid-1"));
    }

    #[test]
    fn default_sandbox_contains_exact_proc_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap = Bwrap::initialize(minimal_spec(tempdir.path(), false)).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);
        let observed: Vec<&str> = args
            .windows(3)
            .filter(|window| window[0] == "--ro-bind-try" && window[1] == "/dev/null")
            .map(|window| window[2].as_str())
            .filter(|target| target.starts_with("/proc/"))
            .collect();

        assert_eq!(observed, EXPECTED_PROC_MASKS);
        assert!(args.windows(2).any(|window| window == ["--proc", "/proc"]));
    }

    #[test]
    fn requested_proc_metadata_omits_proc_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .provide_proc_metadata(true)
            .build();
        let bwrap = Bwrap::initialize(spec).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);

        for target in EXPECTED_PROC_MASKS {
            assert!(bind_source_for(&args, target).is_none(), "unexpected mask for {target}");
        }
        assert!(args.windows(2).any(|window| window == ["--proc", "/proc"]));
    }

    #[test]
    fn proc_masks_keep_proc_self_accessible() {
        for target in EXPECTED_PROC_MASKS {
            assert!(!target.starts_with("/proc/self/"));
        }
        assert!(!PROC_METADATA_MASK_ARGS.iter().any(|arg| arg.starts_with("/proc/self/")));
    }

    #[test]
    fn default_sandbox_contains_random_device_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap = Bwrap::initialize(minimal_spec(tempdir.path(), false)).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);
        let masked = ["/dev/random", "/dev/urandom"];

        for path in &masked {
            assert_eq!(bind_source_for(&args, path).as_deref(), Some("/dev/null"), "/dev mask missing for {path}");
        }
    }

    #[test]
    fn requested_random_devices_omit_random_device_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .provide_random_devices(true)
            .build();
        let bwrap = Bwrap::initialize(spec).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);

        assert!(bind_source_for(&args, "/dev/random").is_none());
        assert!(bind_source_for(&args, "/dev/urandom").is_none());
    }

    #[test]
    fn common_bwrap_args_contains_dev_shm_isolation() {
        assert!(COMMON_BWRAP_ARGS.windows(2).any(|w| w == ["--tmpfs", "/dev/shm"]));
    }

    #[test]
    fn common_bwrap_args_contains_cgroup_unshare() {
        assert!(COMMON_BWRAP_ARGS.contains(&"--unshare-cgroup-try"));
    }

    #[test]
    fn common_bwrap_args_does_not_mount_sys() {
        for window in COMMON_BWRAP_ARGS.windows(2) {
            let bind_types = ["--bind", "--ro-bind", "--ro-bind-try", "--dev-bind"];
            if bind_types.contains(&window[0]) {
                assert!(!window[1].starts_with("/sys"), "must not bind-mount host /sys: found {:?}", window);
            }
        }
    }

    #[test]
    fn synthetic_etc_services_is_not_empty() {
        assert!(!ETC_SERVICES.is_empty());
        assert!(ETC_SERVICES.windows("https".len()).any(|w| w == b"https"));
    }

    #[test]
    fn network_sandbox_binds_explicit_private_resolver_without_public_dns_fallback() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let dns = crate::buildservice::SandboxDnsConfig::new(vec!["10.53.0.2".parse().expect("valid ip")], vec![
            "corp.internal".to_string(),
        ])
        .expect("valid private dns config");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .allow_network(true)
            .dns_config(Some(dns))
            .build();
        let bwrap = Bwrap::initialize(spec).expect("network sandbox with explicit resolver should initialize");
        let args = arg_strings(&bwrap);

        let resolv_source = bind_source_for(&args, "/etc/resolv.conf").expect("resolv.conf must be bound");
        let services_source = bind_source_for(&args, "/etc/services").expect("services must be bound");
        assert_ne!(resolv_source, "/etc/resolv.conf");
        assert_ne!(services_source, "/etc/services");
        let resolv_contents = fs::read_to_string(&resolv_source).expect("synthetic resolv.conf readable");
        assert_eq!(resolv_contents, "nameserver 10.53.0.2\nsearch corp.internal\n");
        assert!(!resolv_contents.contains("8.8.8.8"));
        assert!(!resolv_contents.contains("127.0.0.1"));
        assert_eq!(fs::read(&services_source).expect("synthetic services readable"), ETC_SERVICES);
    }

    #[test]
    fn network_sandbox_without_explicit_dns_config_fails_closed() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let error = match Bwrap::initialize(minimal_spec(tempdir.path(), true)) {
            Ok(_) => panic!("network sandbox without DNS config must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert!(error.to_string().contains("DNS resolver configuration"));
    }

    #[test]
    fn non_network_sandbox_omits_resolv_conf_and_services_and_rejects_dns_config() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap =
            Bwrap::initialize(minimal_spec(tempdir.path(), false)).expect("non-network sandbox should initialize");
        let args = arg_strings(&bwrap);

        assert!(args.iter().any(|arg| arg == "--unshare-net"));
        assert!(bind_source_for(&args, "/etc/resolv.conf").is_none());
        assert!(bind_source_for(&args, "/etc/services").is_none());
        assert!(bind_source_for(&args, "/etc/nsswitch.conf").is_none());
        assert!(!tempdir.path().join("etc/resolv.conf").exists());
        assert!(!tempdir.path().join("etc/services").exists());

        let tempdir2 = tempfile::tempdir().expect("tempdir must be available");
        let dns = crate::buildservice::SandboxDnsConfig::new(vec!["10.53.0.2".parse().expect("valid ip")], Vec::new())
            .expect("valid dns");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir2.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .allow_network(false)
            .dns_config(Some(dns))
            .build();
        let error = match Bwrap::initialize(spec) {
            Ok(_) => panic!("offline sandbox with DNS config must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert!(error.to_string().contains("forbidden"));
    }

    #[test]
    fn sandbox_dns_config_rejects_malformed_and_unbounded_inputs() {
        use crate::buildservice::MAX_DNS_NAMESERVERS;
        use crate::buildservice::MAX_DNS_RESOLV_CONF_BYTES;
        use crate::buildservice::MAX_DNS_SEARCH_DOMAINS;
        use crate::buildservice::SandboxDnsConfig;

        assert!(SandboxDnsConfig::new(Vec::new(), Vec::new()).is_err());
        let too_many_ns: Vec<std::net::IpAddr> =
            (1..=(MAX_DNS_NAMESERVERS + 1)).map(|i| format!("10.0.0.{i}").parse().unwrap()).collect();
        assert!(SandboxDnsConfig::new(too_many_ns, Vec::new()).is_err());

        let too_many_search: Vec<String> = (0..=MAX_DNS_SEARCH_DOMAINS).map(|i| format!("d{i}.internal")).collect();
        assert!(SandboxDnsConfig::new(vec!["10.0.0.1".parse().unwrap()], too_many_search).is_err());

        assert!(SandboxDnsConfig::new(vec!["10.0.0.1".parse().unwrap()], vec!["bad space".into()]).is_err());
        assert!(SandboxDnsConfig::new(vec!["10.0.0.1".parse().unwrap()], vec!["-bad.internal".into()]).is_err());
        assert!(SandboxDnsConfig::parse_resolv_conf("nameserver not-an-ip\n").is_err());
        assert!(SandboxDnsConfig::parse_resolv_conf("unknown directive\n").is_err());
        assert!(SandboxDnsConfig::parse_resolv_conf(&" ".repeat(MAX_DNS_RESOLV_CONF_BYTES + 1)).is_err());

        let tempdir = tempfile::tempdir().expect("tempdir");
        let host_resolv = tempdir.path().join("resolv.conf");
        fs::write(&host_resolv, "# comment\nnameserver 192.0.2.53\nsearch lab.local\n").unwrap();
        let parsed = SandboxDnsConfig::from_host_resolv_conf(&host_resolv).expect("valid host resolv snapshot");
        assert_eq!(parsed.render_resolv_conf().unwrap(), "nameserver 192.0.2.53\nsearch lab.local\n");
    }
}
