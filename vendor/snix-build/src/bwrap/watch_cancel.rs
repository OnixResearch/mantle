//! Owned local sandbox supervisor with pidfd-backed whole-tree teardown,
//! monotonic wall-clock/silence deadlines, finite-grace shutdown, and cgroup
//! migration/drain coordination.
//!
//! Opening a pidfd for the *inner namespace PID1*, not only the outer bwrap PID,
//! binds the process identity before releasing the start gate. A discovery
//! deadline is diagnostic; unresolved ownership remains gated or quarantined.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use nix::libc;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::time::Instant as TokioInstant;
use tokio::time::sleep;

use super::capture;
use super::cgroup::AttemptCgroup;
use super::cgroup::PendingCgroupSpawn;
use crate::buildservice::SandboxCpuRetirementReason;
use crate::buildservice::SandboxCpuSampleEvent;
use crate::buildservice::SandboxCpuSampleKind;
use crate::buildservice::SandboxCpuScopeAvailability;
use crate::buildservice::SandboxFailureClass;
use crate::buildservice::SandboxLiveCpuAttemptFacts;
use crate::buildservice::SandboxLiveCpuUnavailableReason;
use crate::buildservice::SandboxResourceEnforcementFacts;
use crate::buildservice::SandboxUnavailableReason;
use crate::buildservice::classified_io_error;

const PID1_DISCOVERY_REPORT_DEADLINE: Duration = Duration::from_secs(5);
const PID1_DISCOVERY_POLL: Duration = Duration::from_millis(10);
const TEARDOWN_WAIT: Duration = Duration::from_secs(5);

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum WatchFault {
    PidfdUnavailable,
    DelayPid1Discovery,
    PanicAfterPid1,
    PanicAfterAbort,
    ExitBeforePid1,
}

/// The last owner restores sandbox-created 0000 directories before
/// TempDir cleanup. Do not follow symlinks out of this temporary workspace.
#[derive(Debug)]
pub(crate) struct WatchWorkspace {
    dir: Option<tempfile::TempDir>,
}

impl WatchWorkspace {
    pub(crate) fn new(dir: tempfile::TempDir) -> Self {
        Self { dir: Some(dir) }
    }

    pub(crate) fn path(&self) -> &Path {
        self.dir.as_ref().expect("watch workspace not retained yet").path()
    }

    pub(crate) fn into_tempdir(mut self) -> tempfile::TempDir {
        self.dir.take().expect("watch workspace not retained twice")
    }
}

impl Drop for WatchWorkspace {
    fn drop(&mut self) {
        if let Some(dir) = &self.dir
            && let Err(error) = restore_directory_access(dir.path())
        {
            tracing::warn!(?error, sandbox_path = %dir.path().display(), "watch sandbox cleanup access not restored");
        }
    }
}

fn restore_directory_access(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_dir() {
        return Ok(());
    }
    let mut permissions = metadata.permissions();
    let mode = permissions.mode();
    if mode & 0o700 != 0o700 {
        permissions.set_mode(mode | 0o700);
        fs::set_permissions(path, permissions)?;
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            restore_directory_access(&entry.path())?;
        }
    }
    Ok(())
}

/// Admission, inputs mount guard, and workspace cleanup cannot outlive this
/// spawned sandbox. Fields are ordered so FUSE/inputs unmount happens BEFORE
/// workspace removal, which happens BEFORE concurrency permit release.
pub(crate) struct WatchLease {
    _inputs_guard: Mutex<Option<Box<dyn crate::sandbox::InputsGuard>>>,
    _workspace: Option<Arc<WatchWorkspace>>,
    _permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl WatchLease {
    pub(crate) fn new(permit: tokio::sync::OwnedSemaphorePermit, workspace: Arc<WatchWorkspace>) -> Self {
        Self {
            _inputs_guard: Mutex::new(None),
            _workspace: Some(workspace),
            _permit: Some(permit),
        }
    }

    pub(crate) fn from_inputs_only(inputs_guard: Box<dyn crate::sandbox::InputsGuard>) -> Self {
        Self {
            _inputs_guard: Mutex::new(Some(inputs_guard)),
            _workspace: None,
            _permit: None,
        }
    }

    pub(crate) fn attach_inputs_guard(&mut self, inputs_guard: Box<dyn crate::sandbox::InputsGuard>) {
        *self._inputs_guard.get_mut().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(inputs_guard);
    }
}

/// An outer bwrap/wrapper process identity pinned *before* inspecting `/proc` children.
pub(crate) struct OwnedOuterBwrap {
    pid: u32,
    pidfd: OwnedFd,
}

/// Reject a sandbox *before spawning* if the kernel cannot create pidfds.
/// A successful self-pidfd does not grant cancellation authority over other
/// processes; it only establishes this kernel capability for the later owned
/// bwrap/namespace PID1 descriptors.
pub(crate) fn require_pidfd_support() -> io::Result<()> {
    // SAFETY: getpid supplies this process identity; pidfd_open returns a new
    // owned descriptor on success, closed before any sandbox is started.
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, std::process::id(), 0) };
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful pidfd_open returned a unique owned descriptor.
    drop(unsafe { OwnedFd::from_raw_fd(raw as i32) });
    Ok(())
}

impl OwnedOuterBwrap {
    pub(crate) fn open_for_spawned_child(pid: u32) -> io::Result<Self> {
        // SAFETY: Tokio retains its Child handle and does not reap it before
        // this call. A zombie cannot have its numeric PID reused meanwhile.
        let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: pidfd_open returned an independently owned descriptor.
        Ok(Self {
            pid,
            pidfd: unsafe { OwnedFd::from_raw_fd(raw as i32) },
        })
    }

    fn is_exited(&self) -> io::Result<bool> {
        let mut watched = libc::pollfd {
            fd: self.pidfd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: watched is one valid pollfd and self owns its fd.
        let status = unsafe { libc::poll(&mut watched, 1, 0) };
        if status < 0 {
            return Err(io::Error::last_os_error());
        }
        if status > 0 && watched.revents & libc::POLLERR != 0 {
            return Err(io::Error::other("outer bwrap pidfd poll failed"));
        }
        Ok(status > 0 && watched.revents & libc::POLLIN != 0)
    }
}

pub(crate) struct OwnedPidNamespaceLeader {
    pidfd: OwnedFd,
}

impl OwnedPidNamespaceLeader {
    /// Discover only the namespace PID1 owned by a *pinned, live* outer
    /// process (depth 1: `outer -> pid1`, or depth 2: `outer -> bwrap -> pid1`
    /// when wrapped by `BwrapCommandFactory` / `ProtectedBuildRuntime`).
    pub(crate) fn open_from_outer(outer: &OwnedOuterBwrap) -> io::Result<Self> {
        if outer.is_exited()? {
            return Err(io::Error::new(ErrorKind::WouldBlock, "outer bwrap exited before PID1 ownership"));
        }
        let outer_pid = outer.pid;
        let outer_pid_ns = fs::read_link(format!("/proc/{outer_pid}/ns/pid"))?;
        let children = fs::read_to_string(format!("/proc/{outer_pid}/task/{outer_pid}/children"))?;
        for child in children.split_whitespace().take(32) {
            let child_pid = child.parse::<u32>().map_err(|_| io::Error::other("invalid bwrap child pid"))?;
            if owned_namespace_leader(outer_pid, child_pid, &outer_pid_ns).unwrap_or(false) {
                return Self::bind_verified_leader(outer, outer_pid, child_pid, &outer_pid_ns);
            }
            if is_direct_child(outer_pid, child_pid).unwrap_or(false)
                && let Ok(grandchildren) = fs::read_to_string(format!("/proc/{child_pid}/task/{child_pid}/children"))
            {
                for grandchild in grandchildren.split_whitespace().take(32) {
                    let grandchild_pid =
                        grandchild.parse::<u32>().map_err(|_| io::Error::other("invalid bwrap grandchild pid"))?;
                    if owned_namespace_leader(child_pid, grandchild_pid, &outer_pid_ns).unwrap_or(false) {
                        return Self::bind_verified_leader(outer, child_pid, grandchild_pid, &outer_pid_ns);
                    }
                }
            }
        }
        Err(io::Error::new(ErrorKind::WouldBlock, "outer bwrap has no observed namespace PID1"))
    }

    fn bind_verified_leader(
        outer: &OwnedOuterBwrap,
        expected_parent_pid: u32,
        leader_pid: u32,
        outer_pid_ns: &std::path::Path,
    ) -> io::Result<Self> {
        // SAFETY: pidfd_open is a Linux syscall returning a new owned file
        // descriptor on success. No process identity is recovered from a
        // stale numeric PID after this point.
        let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, leader_pid, 0) };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: a successful pidfd_open returns a unique owned fd.
        let pidfd = unsafe { OwnedFd::from_raw_fd(raw as i32) };
        // Close an fd opened during parentage/PID namespace turnover rather
        // than misidentifying an unrelated process.
        if !outer.is_exited()?
            && (expected_parent_pid == outer.pid || is_direct_child(outer.pid, expected_parent_pid).unwrap_or(false))
            && owned_namespace_leader(expected_parent_pid, leader_pid, outer_pid_ns).unwrap_or(false)
        {
            return Ok(Self { pidfd });
        }
        Err(io::Error::new(ErrorKind::WouldBlock, "namespace leader changed during ownership check"))
    }

    /// Send SIGTERM to the owned PID namespace leader (used for graceful shutdown
    /// before escalating to SIGKILL after the finite grace period).
    pub(crate) fn send_sigterm(&self) -> io::Result<()> {
        // SAFETY: pidfd_send_signal targets this previously opened owned fd.
        let sent = unsafe { libc::syscall(libc::SYS_pidfd_send_signal, self.pidfd.as_raw_fd(), libc::SIGTERM, 0, 0) };
        if sent < 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// SIGKILL the owned PID namespace leader and observe its actual exit.
    /// Linux kills all members of that PID namespace when its init exits,
    /// including children that escaped the original process group via setsid.
    pub(crate) fn terminate_and_wait(&self, deadline: Duration) -> io::Result<()> {
        // SAFETY: pidfd_send_signal targets this previously opened owned fd,
        // not a reused numeric PID. A zero siginfo pointer is supported.
        let sent = unsafe { libc::syscall(libc::SYS_pidfd_send_signal, self.pidfd.as_raw_fd(), libc::SIGKILL, 0, 0) };
        if sent < 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err(io::Error::last_os_error());
        }
        self.wait_pollin(deadline)
    }

    fn wait_pollin(&self, deadline: Duration) -> io::Result<()> {
        let until = Instant::now() + deadline;
        loop {
            let mut watched = libc::pollfd {
                fd: self.pidfd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let remaining = until.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(io::Error::new(ErrorKind::TimedOut, "sandbox PID1 teardown unobserved"));
            }
            let timeout = i32::try_from(remaining.as_millis()).unwrap_or(i32::MAX).max(1);
            // SAFETY: watched points to one live stack pollfd and the pidfd
            // remains owned by self for the entire call.
            let result = unsafe { libc::poll(&mut watched, 1, timeout) };
            if result > 0 && watched.revents & libc::POLLIN != 0 {
                return Ok(());
            }
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() != ErrorKind::Interrupted {
                    return Err(error);
                }
            } else if result > 0 && watched.revents & libc::POLLERR != 0 {
                return Err(io::Error::other("pidfd poll failed before namespace teardown"));
            }
        }
    }
}

fn is_direct_child(parent_pid: u32, child_pid: u32) -> io::Result<bool> {
    let status = fs::read_to_string(format!("/proc/{child_pid}/status"))?;
    let parent = status
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .and_then(|text| text.trim().parse::<u32>().ok());
    Ok(parent == Some(parent_pid))
}

fn owned_namespace_leader(parent_pid: u32, child_pid: u32, outer_pid_ns: &Path) -> io::Result<bool> {
    let status = fs::read_to_string(format!("/proc/{child_pid}/status"))?;
    let parent = status
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .and_then(|text| text.trim().parse::<u32>().ok());
    let innermost_pid = status
        .lines()
        .find_map(|line| line.strip_prefix("NSpid:"))
        .and_then(|text| text.split_whitespace().next_back())
        .and_then(|text| text.parse::<u32>().ok());
    if parent != Some(parent_pid) || innermost_pid != Some(1) {
        return Ok(false);
    }
    let child_pid_ns = fs::read_link(format!("/proc/{child_pid}/ns/pid"))?;
    Ok(child_pid_ns != outer_pid_ns)
}

/// The builder cannot exec while this owned write end remains unreleased.
/// Bubblewrap `--block-fd` already forked its namespace PID1 at this point;
/// the control pipe's EOF also releases the builder, so never drop this gate
/// until PID1 is bound or observed stopped.
pub(crate) struct WatchStartGate {
    reader: Option<OwnedFd>,
    writer: OwnedFd,
}

impl WatchStartGate {
    pub(crate) fn new() -> io::Result<Self> {
        let mut fds = [-1_i32; 2];
        // SAFETY: pipe2 fills exactly two caller-owned fds or fails without
        // transferring ownership. Both endpoints start close-on-exec.
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: a successful pipe2 returned two independent owned fds.
        Ok(Self {
            reader: Some(unsafe { OwnedFd::from_raw_fd(fds[0]) }),
            writer: unsafe { OwnedFd::from_raw_fd(fds[1]) },
        })
    }

    pub(crate) fn block_arguments(&self) -> [OsString; 2] {
        let fd = self.reader.as_ref().expect("gate reader is configured before spawn").as_raw_fd();
        ["--block-fd".into(), fd.to_string().into()]
    }

    pub(crate) fn configure(&self, command: &mut Command) {
        let fd = self.reader.as_ref().expect("gate reader is configured before spawn").as_raw_fd();
        // SAFETY: pre_exec runs in the forked child immediately before exec.
        // fcntl is async-signal-safe and touches only this already-open fd.
        unsafe {
            command.pre_exec(move || {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    /// The parent's duplicate otherwise makes reader-exit detection impossible.
    fn close_parent_reader(&mut self) {
        self.reader.take();
    }

    /// No reader can receive this writer's eventual byte or EOF.
    fn no_readers(&self) -> io::Result<bool> {
        writer_no_readers(&self.writer)
    }

    fn release(&self) -> io::Result<()> {
        let byte = [1_u8];
        // SAFETY: writer remains owned while this one-byte write executes.
        // Retain the writer on failure: closing it produces EOF, which also
        // releases bubblewrap's blocked payload.
        let written = unsafe { libc::write(self.writer.as_raw_fd(), byte.as_ptr().cast(), byte.len()) };
        if written == 1 {
            Ok(())
        } else if written < 0 {
            Err(io::Error::last_os_error())
        } else {
            Err(io::Error::other("bubblewrap watch start gate accepted no byte"))
        }
    }
}

fn writer_no_readers(writer: &OwnedFd) -> io::Result<bool> {
    let mut watched = libc::pollfd {
        fd: writer.as_raw_fd(),
        events: libc::POLLOUT,
        revents: 0,
    };
    // SAFETY: writer is owned for this entire poll.
    let status = unsafe { libc::poll(&mut watched, 1, 0) };
    if status < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(status > 0 && watched.revents & libc::POLLERR != 0)
}

struct AbortOnDrop(Option<oneshot::Sender<()>>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

#[derive(Default)]
struct SupervisorIdentities {
    outer: Option<Arc<OwnedOuterBwrap>>,
    pid1: Option<Arc<OwnedPidNamespaceLeader>>,
}

pub(crate) fn monotonic_now_usec() -> u64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `ts` is a valid stack `timespec` passed to `CLOCK_MONOTONIC`.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) } == 0 {
        let secs = u64::try_from(ts.tv_sec).unwrap_or(0);
        let nsec = u64::try_from(ts.tv_nsec).unwrap_or(0);
        secs.saturating_mul(1_000_000).saturating_add(nsec / 1_000)
    } else {
        0
    }
}

pub(crate) struct SupervisorLiveCpuConfig {
    pub(crate) attempt_id: u64,
    pub(crate) generation: u64,
    pub(crate) attempt_label: String,
    pub(crate) allocated_cores: u32,
    pub(crate) cadence: Duration,
    pub(crate) sender: tokio::sync::mpsc::Sender<SandboxCpuSampleEvent>,
    pub(crate) facts_sink: Option<Arc<Mutex<Option<SandboxLiveCpuAttemptFacts>>>>,
    pub(crate) initial_unavailable: Option<SandboxLiveCpuUnavailableReason>,
    pub(crate) diagnostics_only_cgroup: bool,
}

struct LiveCpuSamplerState {
    attempt_id: u64,
    generation: u64,
    attempt_label: String,
    allocated_cores: u32,
    sender: Option<tokio::sync::mpsc::Sender<SandboxCpuSampleEvent>>,
    facts_sink: Option<Arc<Mutex<Option<SandboxLiveCpuAttemptFacts>>>>,
    scope: SandboxCpuScopeAvailability,
    sampling_active: bool,
    started_emitted: bool,
    retired: bool,
    sample_count: u64,
    dropped_samples_count: u64,
    initial_cpu_usec: Option<u64>,
    latest_cpu_usec: Option<u64>,
    retired_reason: Option<SandboxCpuRetirementReason>,
}

impl LiveCpuSamplerState {
    fn from_config(cfg: SupervisorLiveCpuConfig) -> Self {
        let scope = match cfg.initial_unavailable {
            Some(reason) => SandboxCpuScopeAvailability::Unavailable { reason },
            None => SandboxCpuScopeAvailability::Unavailable {
                reason: SandboxLiveCpuUnavailableReason::CgroupRootUnconfigured,
            },
        };
        Self {
            attempt_id: cfg.attempt_id,
            generation: cfg.generation,
            attempt_label: cfg.attempt_label,
            allocated_cores: cfg.allocated_cores,
            sender: Some(cfg.sender),
            facts_sink: cfg.facts_sink,
            scope,
            sampling_active: false,
            started_emitted: false,
            retired: false,
            sample_count: 0,
            dropped_samples_count: 0,
            initial_cpu_usec: None,
            latest_cpu_usec: None,
            retired_reason: None,
        }
    }

    fn publish_facts(&self) {
        if let Some(sink) = &self.facts_sink {
            *sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(SandboxLiveCpuAttemptFacts {
                attempt_id: self.attempt_id,
                generation: self.generation,
                attempt_label: self.attempt_label.clone(),
                scope: self.scope.clone(),
                sample_count: self.sample_count,
                dropped_samples_count: self.dropped_samples_count,
                initial_cpu_usec: self.initial_cpu_usec,
                latest_cpu_usec: self.latest_cpu_usec,
                retired_reason: self.retired_reason,
            });
        }
    }

    fn try_emit(&mut self, kind: SandboxCpuSampleKind) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };
        let event = SandboxCpuSampleEvent {
            attempt_id: self.attempt_id,
            generation: self.generation,
            attempt_label: self.attempt_label.clone(),
            allocated_cores: self.allocated_cores,
            observed_at_usec: monotonic_now_usec(),
            kind,
        };
        match sender.try_send(event) {
            Ok(()) => {}
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                self.dropped_samples_count = self.dropped_samples_count.saturating_add(1);
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                self.dropped_samples_count = self.dropped_samples_count.saturating_add(1);
                self.sender = None;
                self.sampling_active = false;
            }
        }
    }

    fn mark_started_with_cgroup(&mut self, cgroup: &AttemptCgroup) {
        if self.retired || self.started_emitted {
            return;
        }
        self.scope = SandboxCpuScopeAvailability::OwnedAttemptCgroupV2 {
            rel_path: cgroup.rel_path().to_string(),
        };
        self.sampling_active = true;
        self.started_emitted = true;
        self.try_emit(SandboxCpuSampleKind::Started {
            scope: self.scope.clone(),
        });
        self.sample_once(cgroup);
        self.publish_facts();
    }

    fn mark_started_unavailable(&mut self, reason: SandboxLiveCpuUnavailableReason) {
        if self.retired || self.started_emitted {
            return;
        }
        self.scope = SandboxCpuScopeAvailability::Unavailable { reason };
        self.sampling_active = false;
        self.started_emitted = true;
        self.try_emit(SandboxCpuSampleKind::Started {
            scope: self.scope.clone(),
        });
        self.publish_facts();
    }

    fn sample_once(&mut self, cgroup: &AttemptCgroup) {
        if self.retired || !self.sampling_active {
            return;
        }
        match cgroup.read_cpu_usage_usec() {
            Ok(usage_usec) => {
                if self.initial_cpu_usec.is_none() {
                    self.initial_cpu_usec = Some(usage_usec);
                }
                self.latest_cpu_usec = Some(usage_usec);
                self.sample_count = self.sample_count.saturating_add(1);
                self.try_emit(SandboxCpuSampleKind::Sample {
                    cumulative_cpu_usec: usage_usec,
                });
            }
            Err(err) => {
                self.try_emit(SandboxCpuSampleKind::Unreadable {
                    detail: err.to_string(),
                });
            }
        }
        self.publish_facts();
    }

    fn retire(&mut self, reason: SandboxCpuRetirementReason) {
        if self.retired {
            return;
        }
        self.retired = true;
        self.sampling_active = false;
        self.retired_reason = Some(reason);
        self.try_emit(SandboxCpuSampleKind::Retired { reason });
        self.sender = None;
        self.publish_facts();
    }
}

fn retire_live_cpu_sampler(sampler: &Option<Arc<Mutex<LiveCpuSamplerState>>>, reason: SandboxCpuRetirementReason) {
    if let Some(state) = sampler {
        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.retire(reason);
    }
}

pub(crate) struct SupervisorRunConfig {
    pub(crate) wall_deadline: Option<Duration>,
    pub(crate) silence_budget: Option<Duration>,
    pub(crate) shutdown_grace: Duration,
    pub(crate) shutdown_signal: Option<watch::Receiver<bool>>,
    pub(crate) cgroup: Option<Arc<AttemptCgroup>>,
    pub(crate) cgroup_ack_reader: Option<std::os::unix::net::UnixStream>,
    pub(crate) pending_cgroup_spawn: Option<PendingCgroupSpawn>,
    pub(crate) enforcement_facts: Arc<Mutex<SandboxResourceEnforcementFacts>>,
    pub(crate) live_cpu: Option<SupervisorLiveCpuConfig>,
}

/// The supervisor, not the awaiting BuildService future, owns the child and
/// startup gate. Dropping the future requests cancellation without closing
/// the gate or losing the PID1/outer/cgroup reaping obligation.
pub(crate) async fn run_child(
    mut command: Command,
    program: &OsString,
    cancellation: Option<watch::Receiver<bool>>,
    mut gate: WatchStartGate,
    lease: WatchLease,
    mut config: SupervisorRunConfig,
    #[cfg(test)] fault: Option<WatchFault>,
) -> io::Result<std::process::Output> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let diagnostics_only_cgroup = config.live_cpu.as_ref().is_some_and(|cfg| cfg.diagnostics_only_cgroup);
    let live_cpu_cadence = config.live_cpu.as_ref().map(|cfg| cfg.cadence);
    let live_cpu_initial_unavailable = config.live_cpu.as_ref().and_then(|cfg| cfg.initial_unavailable.clone());
    let live_cpu_sampler =
        config.live_cpu.take().map(|cfg| Arc::new(Mutex::new(LiveCpuSamplerState::from_config(cfg))));
    // Make the emergency writer while no child exists; a failed dup must not
    // strand a spawned sandbox. A supervisor panic cannot create gate EOF.
    let panic_writer = gate.writer.try_clone()?;
    let child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            drop(config.pending_cgroup_spawn.take());
            if config.cgroup.is_some() {
                let mut pre_exec_migration_failed = false;
                if let Some(mut ack_reader) = config.cgroup_ack_reader.take() {
                    let mut byte = [0_u8; 1];
                    if matches!(std::io::Read::read(&mut ack_reader, &mut byte), Ok(1)) && byte[0] == b'0' {
                        pre_exec_migration_failed = true;
                    }
                }
                let mut facts = config.enforcement_facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                facts.enforced_memory = false;
                facts.enforced_pids = false;
                if pre_exec_migration_failed {
                    let reason = SandboxUnavailableReason::ChildMigrationFailed {
                        detail: format!("child pre_exec cgroup migration failed: {error}"),
                    };
                    facts.unavailable_reasons.push(reason);
                    if let Some(state) = live_cpu_sampler.as_ref() {
                        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        guard.mark_started_unavailable(SandboxLiveCpuUnavailableReason::ChildMigrationFailed {
                            detail: format!("child pre_exec cgroup migration failed: {error}"),
                        });
                        guard.retire(SandboxCpuRetirementReason::Failed);
                    }
                    return Err(classified_io_error(
                        error.kind(),
                        SandboxFailureClass::ResourceEnforcementUnavailable,
                        format!("cgroup migration failed during spawn: {error}"),
                    ));
                }
            }
            if let Some(state) = live_cpu_sampler.as_ref() {
                let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.mark_started_unavailable(SandboxLiveCpuUnavailableReason::SetupIoError {
                    detail: format!("sandbox spawn failed: {error}"),
                });
                guard.retire(SandboxCpuRetirementReason::Failed);
            }
            return Err(super::annotate_bwrap_spawn_error(program, error));
        }
    };
    drop(config.pending_cgroup_spawn.take());
    gate.close_parent_reader();
    let lease = Arc::new(lease);
    let identities = Arc::new(Mutex::new(SupervisorIdentities::default()));
    let (abort_tx, abort_rx) = oneshot::channel();
    let watchdog_cgroup = config.cgroup.clone();
    let watchdog_facts = Arc::clone(&config.enforcement_facts);
    let watchdog_sampler = live_cpu_sampler.clone();
    let worker = tokio::spawn(supervise_child(SupervisedChildContext {
        child,
        gate,
        cancellation,
        abort: abort_rx,
        lease: Arc::clone(&lease),
        identities: Arc::clone(&identities),
        config,
        live_cpu_sampler,
        live_cpu_cadence,
        live_cpu_initial_unavailable,
        diagnostics_only_cgroup,
        #[cfg(test)]
        fault,
    }));
    // This watchdog, not the abortable caller, owns the emergency writer and
    // admission lease until even a panicking sandbox supervisor is resolved.
    let watchdog = tokio::spawn(async move {
        match worker.await {
            Ok(result) => {
                retire_live_cpu_sampler(&watchdog_sampler, SandboxCpuRetirementReason::Completed);
                drop(watchdog_cgroup);
                result
            }
            Err(error) => {
                retire_live_cpu_sampler(&watchdog_sampler, SandboxCpuRetirementReason::Failed);
                let (pid1, outer) = {
                    let owned = identities.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    (owned.pid1.clone(), owned.outer.clone())
                };
                if let (Some(pid1), Some(outer)) = (pid1, outer.as_ref()) {
                    terminate_until_observed(&pid1).await;
                    wait_outer_exit(outer).await;
                    let _ = finalize_cgroup(watchdog_cgroup, &watchdog_facts).await;
                    return Err(io::Error::other(format!("watch supervisor failed after owned teardown: {error}")));
                }
                if let Some(outer) = outer.as_ref() {
                    let until = TokioInstant::now() + PID1_DISCOVERY_REPORT_DEADLINE;
                    loop {
                        if let Ok(pid1) = OwnedPidNamespaceLeader::open_from_outer(outer) {
                            terminate_until_observed(&Arc::new(pid1)).await;
                            wait_outer_exit(outer).await;
                            let _ = finalize_cgroup(watchdog_cgroup, &watchdog_facts).await;
                            return Err(io::Error::other(format!(
                                "watch supervisor failed; recovered PID1 teardown: {error}"
                            )));
                        }
                        if outer.is_exited().unwrap_or(false) {
                            if writer_no_readers(&panic_writer).unwrap_or(false) {
                                let _ = finalize_cgroup(watchdog_cgroup, &watchdog_facts).await;
                                return Err(io::Error::other(format!(
                                    "watch supervisor failed after terminal bwrap exit: {error}"
                                )));
                            }
                            break;
                        }
                        if TokioInstant::now() >= until {
                            break;
                        }
                        sleep(PID1_DISCOVERY_POLL).await;
                    }
                } else if writer_no_readers(&panic_writer).unwrap_or(false) {
                    let _ = finalize_cgroup(watchdog_cgroup, &watchdog_facts).await;
                    return Err(io::Error::other(format!(
                        "watch supervisor failed without an inherited gate reader: {error}"
                    )));
                }
                tokio::spawn(async move {
                    loop {
                        let outer_exited = outer.as_ref().is_none_or(|outer| outer.is_exited().unwrap_or(false));
                        if outer_exited && writer_no_readers(&panic_writer).unwrap_or(false) {
                            break;
                        }
                        sleep(PID1_DISCOVERY_POLL).await;
                    }
                    let _ = finalize_cgroup(watchdog_cgroup, &watchdog_facts).await;
                    drop(lease);
                });
                Err(io::Error::other(UnobservedWatchSandbox("supervisor failed before owned PID1 teardown")))
            }
        }
    });
    let mut abort_guard = AbortOnDrop(Some(abort_tx));
    let result = watchdog.await.map_err(|error| {
        io::Error::other(UnobservedWatchSandbox(if error.is_panic() {
            "watchdog panicked"
        } else {
            "watchdog stopped"
        }))
    })?;
    abort_guard.0.take();
    result
}

async fn terminate_until_observed(pid1: &Arc<OwnedPidNamespaceLeader>) {
    loop {
        let held = Arc::clone(pid1);
        match tokio::task::spawn_blocking(move || held.terminate_and_wait(TEARDOWN_WAIT)).await {
            Ok(Ok(())) => return,
            Ok(Err(error)) => tracing::warn!(?error, "watch PID1 teardown still unobserved; reservation remains held"),
            Err(error) => tracing::warn!(?error, "watch PID1 teardown task failed; reservation remains held"),
        }
        sleep(PID1_DISCOVERY_POLL).await;
    }
}

async fn wait_outer_exit(outer: &OwnedOuterBwrap) {
    loop {
        match outer.is_exited() {
            Ok(true) => return,
            Ok(false) => {}
            Err(error) => tracing::warn!(?error, "watch outer bwrap exit unobserved; reservation remains held"),
        }
        sleep(PID1_DISCOVERY_POLL).await;
    }
}

/// An unbound sandbox cannot be declared stopped: its inherited gate reader
/// might still belong to an orphan. Callers must not retry while quarantine
/// retains admission, workspace, and the gate writer.
#[derive(Debug)]
pub struct UnobservedWatchSandbox(&'static str);

impl std::fmt::Display for UnobservedWatchSandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "watch sandbox quarantined; do not retry: {}", self.0)
    }
}

impl std::error::Error for UnobservedWatchSandbox {}

async fn classify_unbound_child(
    mut child: Child,
    gate: WatchStartGate,
    lease: Arc<WatchLease>,
    outer: Option<Arc<OwnedOuterBwrap>>,
    cgroup: Option<Arc<AttemptCgroup>>,
    live_cpu_sampler: Option<Arc<Mutex<LiveCpuSamplerState>>>,
    reason: &'static str,
) -> io::Result<std::process::Output> {
    if let Some(state) = live_cpu_sampler.as_ref() {
        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.mark_started_unavailable(SandboxLiveCpuUnavailableReason::SupervisorPanickedOrUnbound);
        guard.retire(SandboxCpuRetirementReason::Failed);
    }
    let readers_closed = gate.no_readers().unwrap_or(false);
    let outer_exited = if let Some(outer) = &outer {
        outer.is_exited().unwrap_or(false)
    } else {
        child.try_wait().ok().flatten().is_some()
    };
    if readers_closed && outer_exited {
        let output = capture::capture_child_output(child).await?;
        if let Some(cg) = cgroup {
            let _ = tokio::task::spawn_blocking(move || cg.drain_and_collect(TEARDOWN_WAIT)).await;
        }
        return Err(io::Error::other(format!(
            "bubblewrap exited before watch PID1 ownership ({reason}): status={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        )));
    }

    tokio::spawn(async move {
        // Closing the writer while an orphan still owns a reader would start
        // its blocked payload. Keep the permit and workspace quarantined until
        // both the outer child exits and every inherited reader is gone.
        loop {
            let outer_exited = if let Some(outer) = &outer {
                outer.is_exited().unwrap_or(false)
            } else {
                child.try_wait().ok().flatten().is_some()
            };
            if outer_exited && gate.no_readers().unwrap_or(false) {
                let _ = capture::capture_child_output(child).await;
                break;
            }
            sleep(PID1_DISCOVERY_POLL).await;
        }
        if let Some(cg) = cgroup {
            let _ = tokio::task::spawn_blocking(move || cg.drain_and_collect(TEARDOWN_WAIT)).await;
        }
        drop(lease);
    });
    Err(io::Error::other(UnobservedWatchSandbox(reason)))
}

fn is_watch_triggered(rx: &Option<watch::Receiver<bool>>) -> bool {
    rx.as_ref().is_some_and(|r| *r.borrow() || r.has_changed().is_err())
}

async fn wait_for_watch_trigger(rx: &mut Option<watch::Receiver<bool>>) -> bool {
    let Some(receiver) = rx.as_mut() else {
        return std::future::pending().await;
    };
    if *receiver.borrow() {
        return true;
    }
    match receiver.changed().await {
        Ok(()) => *receiver.borrow_and_update(),
        Err(_) => true,
    }
}

async fn finalize_cgroup(
    cgroup: Option<Arc<AttemptCgroup>>,
    facts: &Arc<Mutex<SandboxResourceEnforcementFacts>>,
) -> io::Result<Option<super::cgroup::CgroupPostExitEvents>> {
    let Some(cg) = cgroup else {
        return Ok(None);
    };
    let events = tokio::task::spawn_blocking(move || cg.drain_and_collect(TEARDOWN_WAIT))
        .await
        .map_err(|err| io::Error::other(format!("cgroup drain task failed: {err}")))??;
    let mut guard = facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.oom_count = events.oom_count;
    guard.oom_kill_count = events.oom_kill_count;
    guard.memory_peak_bytes = events.memory_peak_bytes;
    guard.pids_max_events = events.pids_max_events;
    Ok(Some(events))
}

async fn reap_and_drain(
    pid1: &Arc<OwnedPidNamespaceLeader>,
    outer: &Arc<OwnedOuterBwrap>,
    wait_future: &mut std::pin::Pin<&mut impl std::future::Future<Output = io::Result<std::process::Output>>>,
    cgroup: Option<Arc<AttemptCgroup>>,
    facts: &Arc<Mutex<SandboxResourceEnforcementFacts>>,
    live_cpu_sampler: &Option<Arc<Mutex<LiveCpuSamplerState>>>,
    retire_reason: SandboxCpuRetirementReason,
) -> io::Result<Option<std::process::Output>> {
    retire_live_cpu_sampler(live_cpu_sampler, retire_reason);
    terminate_until_observed(pid1).await;
    let output_res = wait_future.as_mut().await;
    if output_res.is_err() {
        wait_outer_exit(outer).await;
    }
    let _ = finalize_cgroup(cgroup, facts).await;
    match output_res {
        Ok(output) => Ok(Some(output)),
        Err(error) => Err(error),
    }
}

fn format_deadline_diagnostic(
    headline: &str,
    budget: Option<Duration>,
    output: Option<&std::process::Output>,
    facts: &Arc<Mutex<SandboxResourceEnforcementFacts>>,
) -> String {
    let mut parts = vec![headline.to_string()];
    if let Some(dur) = budget {
        parts.push(format!("budget_ms={}", dur.as_millis()));
    }
    let guard = facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.oom_kill_count > 0 {
        parts.push(format!("oom_kill={}", guard.oom_kill_count));
    }
    if let Some(out) = output {
        let stderr_tail = String::from_utf8_lossy(&out.stderr);
        let snippet: String = stderr_tail.chars().take(512).collect();
        if !snippet.is_empty() {
            parts.push(format!("stderr={snippet}"));
        }
    }
    parts.join("; ")
}

struct GracefulShutdownContext<'a> {
    pid1: &'a Arc<OwnedPidNamespaceLeader>,
    outer: &'a Arc<OwnedOuterBwrap>,
    abort: &'a mut oneshot::Receiver<()>,
    cancellation: &'a mut Option<watch::Receiver<bool>>,
    wall_deadline_instant: Option<TokioInstant>,
    shutdown_grace: Duration,
    cgroup: Option<Arc<AttemptCgroup>>,
    facts: &'a Arc<Mutex<SandboxResourceEnforcementFacts>>,
    live_cpu_sampler: &'a Option<Arc<Mutex<LiveCpuSamplerState>>>,
}

async fn handle_graceful_shutdown(
    ctx: GracefulShutdownContext<'_>,
    wait_future: &mut std::pin::Pin<&mut impl std::future::Future<Output = io::Result<std::process::Output>>>,
) -> io::Result<std::process::Output> {
    let _ = ctx.pid1.send_sigterm();
    let grace_deadline = TokioInstant::now() + ctx.shutdown_grace;
    let effective_deadline = match ctx.wall_deadline_instant {
        Some(wall) if wall < grace_deadline => wall,
        _ => grace_deadline,
    };
    let grace_sleep = tokio::time::sleep_until(effective_deadline);
    tokio::pin!(grace_sleep);

    loop {
        tokio::select! {
            biased;
            _ = &mut *ctx.abort => {
                reap_and_drain(
                    ctx.pid1,
                    ctx.outer,
                    wait_future,
                    ctx.cgroup,
                    ctx.facts,
                    ctx.live_cpu_sampler,
                    SandboxCpuRetirementReason::DroppedOrAborted,
                )
                .await?;
                return Err(io::Error::new(ErrorKind::Interrupted, "dropped sandbox build stopped during shutdown grace"));
            }
            triggered = wait_for_watch_trigger(ctx.cancellation), if ctx.cancellation.is_some() => {
                if triggered {
                    reap_and_drain(
                        ctx.pid1,
                        ctx.outer,
                        wait_future,
                        ctx.cgroup,
                        ctx.facts,
                        ctx.live_cpu_sampler,
                        SandboxCpuRetirementReason::Cancelled,
                    )
                    .await?;
                    return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox cancelled during shutdown grace"));
                }
            }
            () = &mut grace_sleep => {
                reap_and_drain(
                    ctx.pid1,
                    ctx.outer,
                    wait_future,
                    ctx.cgroup,
                    ctx.facts,
                    ctx.live_cpu_sampler,
                    SandboxCpuRetirementReason::GracefulShutdown,
                )
                .await?;
                return Err(io::Error::new(
                    ErrorKind::Interrupted,
                    format!(
                        "local sandbox killed via SIGKILL after {} ms shutdown grace period expired",
                        ctx.shutdown_grace.as_millis()
                    ),
                ));
            }
            result = wait_future.as_mut() => {
                retire_live_cpu_sampler(ctx.live_cpu_sampler, SandboxCpuRetirementReason::GracefulShutdown);
                terminate_until_observed(ctx.pid1).await;
                if result.is_err() {
                    wait_outer_exit(ctx.outer).await;
                }
                let _ = finalize_cgroup(ctx.cgroup, ctx.facts).await;
                return Err(io::Error::new(
                    ErrorKind::Interrupted,
                    "local sandbox stopped cleanly during shutdown grace period",
                ));
            }
        }
    }
}

struct SupervisedChildContext {
    child: Child,
    gate: WatchStartGate,
    cancellation: Option<watch::Receiver<bool>>,
    abort: oneshot::Receiver<()>,
    lease: Arc<WatchLease>,
    identities: Arc<Mutex<SupervisorIdentities>>,
    config: SupervisorRunConfig,
    live_cpu_sampler: Option<Arc<Mutex<LiveCpuSamplerState>>>,
    live_cpu_cadence: Option<Duration>,
    live_cpu_initial_unavailable: Option<SandboxLiveCpuUnavailableReason>,
    diagnostics_only_cgroup: bool,
    #[cfg(test)]
    fault: Option<WatchFault>,
}

async fn supervise_child(mut ctx: SupervisedChildContext) -> io::Result<std::process::Output> {
    let Some(outer_pid) = ctx.child.id() else {
        return classify_unbound_child(
            ctx.child,
            ctx.gate,
            ctx.lease,
            None,
            ctx.config.cgroup.take(),
            ctx.live_cpu_sampler.take(),
            "spawned child has no PID",
        )
        .await;
    };
    let outer_deadline = TokioInstant::now() + PID1_DISCOVERY_REPORT_DEADLINE;
    let outer = loop {
        match OwnedOuterBwrap::open_for_spawned_child(outer_pid) {
            Ok(owner) => {
                let owner = Arc::new(owner);
                ctx.identities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).outer =
                    Some(Arc::clone(&owner));
                break owner;
            }
            Err(error) => {
                if TokioInstant::now() >= outer_deadline {
                    tracing::error!(?error, "watch outer pidfd unavailable after spawn; classifying gated child");
                    return classify_unbound_child(
                        ctx.child,
                        ctx.gate,
                        ctx.lease,
                        None,
                        ctx.config.cgroup.take(),
                        ctx.live_cpu_sampler.take(),
                        "outer pidfd unavailable",
                    )
                    .await;
                }
                sleep(PID1_DISCOVERY_POLL).await;
            }
        }
    };
    let mut reported_discovery_failure = false;
    #[cfg(test)]
    let delayed_until =
        matches!(ctx.fault, Some(WatchFault::DelayPid1Discovery)).then(|| TokioInstant::now() + Duration::from_secs(6));
    let report_after = TokioInstant::now() + PID1_DISCOVERY_REPORT_DEADLINE;
    let pid1 = loop {
        #[cfg(test)]
        if delayed_until.is_some_and(|deadline| TokioInstant::now() < deadline) {
            if !reported_discovery_failure && TokioInstant::now() >= report_after {
                tracing::error!(
                    "watch namespace PID1 discovery deadline exceeded; start gate and reservation remain held"
                );
                reported_discovery_failure = true;
            }
            sleep(PID1_DISCOVERY_POLL).await;
            continue;
        }
        match OwnedPidNamespaceLeader::open_from_outer(&outer) {
            Ok(owner) => {
                let owner = Arc::new(owner);
                ctx.identities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).pid1 =
                    Some(Arc::clone(&owner));
                break owner;
            }
            Err(error) => {
                if outer.is_exited().unwrap_or(false) {
                    return classify_unbound_child(
                        ctx.child,
                        ctx.gate,
                        ctx.lease,
                        Some(outer),
                        ctx.config.cgroup.take(),
                        ctx.live_cpu_sampler.take(),
                        "outer exited before namespace PID1",
                    )
                    .await;
                }
                if !reported_discovery_failure && TokioInstant::now() >= report_after {
                    tracing::error!(?error, "watch namespace PID1 unobserved; start gate and reservation remain held");
                    reported_discovery_failure = true;
                }
                sleep(PID1_DISCOVERY_POLL).await;
            }
        }
    };
    #[cfg(test)]
    if matches!(ctx.fault, Some(WatchFault::PanicAfterPid1)) {
        panic!("injected supervisor failure after owned PID1 before payload release");
    }
    #[cfg(test)]
    if matches!(ctx.fault, Some(WatchFault::PanicAfterAbort)) {
        if let Some(workspace) = &ctx.lease._workspace {
            fs::write(workspace.path().join("watch-pid1-owned"), b"owned")
                .expect("fault fixture must publish post-PID1 readiness");
        }
        let _ = (&mut ctx.abort).await;
        panic!("injected supervisor failure after caller abort before payload release");
    }

    if let (Some(cg), Some(ack_reader)) = (ctx.config.cgroup.as_ref(), ctx.config.cgroup_ack_reader.take()) {
        let cg_clone = Arc::clone(cg);
        let ack_result = tokio::task::spawn_blocking(move || {
            cg_clone.wait_migration_ack(ack_reader, PID1_DISCOVERY_REPORT_DEADLINE)
        })
        .await
        .map_err(|err| SandboxUnavailableReason::ChildMigrationFailed {
            detail: format!("migration ack task failed: {err}"),
        })
        .and_then(|inner| inner);
        if let Err(reason) = ack_result {
            if ctx.diagnostics_only_cgroup {
                if let Some(state) = ctx.live_cpu_sampler.as_ref() {
                    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.mark_started_unavailable(super::cgroup::unavailable_for_live_cpu(reason));
                }
                drop(ctx.config.cgroup.take());
            } else {
                {
                    let mut facts =
                        ctx.config.enforcement_facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    facts.enforced_memory = false;
                    facts.enforced_pids = false;
                    facts.unavailable_reasons.push(reason.clone());
                }
                if let Some(state) = ctx.live_cpu_sampler.as_ref() {
                    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.mark_started_unavailable(super::cgroup::unavailable_for_live_cpu(reason.clone()));
                    guard.retire(SandboxCpuRetirementReason::Failed);
                }
                terminate_until_observed(&pid1).await;
                let _ = capture::capture_child_output(ctx.child).await;
                wait_outer_exit(&outer).await;
                let _ = finalize_cgroup(ctx.config.cgroup.take(), &ctx.config.enforcement_facts).await;
                return Err(classified_io_error(
                    ErrorKind::Other,
                    SandboxFailureClass::ResourceEnforcementUnavailable,
                    format!("cgroup migration failed before payload start: {reason:?}"),
                ));
            }
        } else {
            let mut facts = ctx.config.enforcement_facts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            cg.observe_live_membership(&mut facts);
            if let Some(state) = ctx.live_cpu_sampler.as_ref() {
                let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.mark_started_with_cgroup(cg);
            }
        }
    } else if let Some(state) = ctx.live_cpu_sampler.as_ref() {
        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let reason = ctx
            .live_cpu_initial_unavailable
            .take()
            .unwrap_or(SandboxLiveCpuUnavailableReason::CgroupRootUnconfigured);
        guard.mark_started_unavailable(reason);
    }

    let activity = Arc::new(capture::OutputActivityTracker::new());
    let wait_future = capture::capture_child_output_with_activity(ctx.child, Some(Arc::clone(&activity)));
    tokio::pin!(wait_future);

    if is_watch_triggered(&ctx.cancellation)
        || is_watch_triggered(&ctx.config.shutdown_signal)
        || ctx.abort.try_recv().is_ok()
    {
        let retire_reason = if is_watch_triggered(&ctx.cancellation) {
            SandboxCpuRetirementReason::Cancelled
        } else if is_watch_triggered(&ctx.config.shutdown_signal) {
            SandboxCpuRetirementReason::GracefulShutdown
        } else {
            SandboxCpuRetirementReason::DroppedOrAborted
        };
        reap_and_drain(
            &pid1,
            &outer,
            &mut wait_future,
            ctx.config.cgroup.take(),
            &ctx.config.enforcement_facts,
            &ctx.live_cpu_sampler,
            retire_reason,
        )
        .await?;
        return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox stopped before builder start"));
    }
    if let Err(error) = ctx.gate.release() {
        let _ = reap_and_drain(
            &pid1,
            &outer,
            &mut wait_future,
            ctx.config.cgroup.take(),
            &ctx.config.enforcement_facts,
            &ctx.live_cpu_sampler,
            SandboxCpuRetirementReason::Failed,
        )
        .await;
        return Err(error);
    }

    let payload_started = TokioInstant::now();
    let wall_deadline_instant = ctx.config.wall_deadline.map(|dur| payload_started + dur);
    let silence_dur = ctx.config.silence_budget;
    let far_future = payload_started + Duration::from_secs(365 * 86_400);
    let wall_sleep = tokio::time::sleep_until(wall_deadline_instant.unwrap_or(far_future));
    tokio::pin!(wall_sleep);
    let silence_sleep = tokio::time::sleep_until(silence_dur.map(|dur| payload_started + dur).unwrap_or(far_future));
    tokio::pin!(silence_sleep);
    let sample_cadence = ctx.live_cpu_cadence.unwrap_or_else(|| Duration::from_millis(500));
    let sample_sleep = tokio::time::sleep_until(payload_started + sample_cadence);
    tokio::pin!(sample_sleep);

    let mut shutdown_signal = ctx.config.shutdown_signal.take();
    loop {
        let sampling_active = ctx.config.cgroup.is_some()
            && ctx
                .live_cpu_sampler
                .as_ref()
                .is_some_and(|state| state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).sampling_active);
        tokio::select! {
            biased;
            _ = &mut ctx.abort => {
                reap_and_drain(
                    &pid1,
                    &outer,
                    &mut wait_future,
                    ctx.config.cgroup.take(),
                    &ctx.config.enforcement_facts,
                    &ctx.live_cpu_sampler,
                    SandboxCpuRetirementReason::DroppedOrAborted,
                )
                .await?;
                return Err(io::Error::new(ErrorKind::Interrupted, "dropped watch build stopped by supervisor"));
            }
            triggered = wait_for_watch_trigger(&mut ctx.cancellation), if ctx.cancellation.is_some() => {
                if triggered {
                    reap_and_drain(
                        &pid1,
                        &outer,
                        &mut wait_future,
                        ctx.config.cgroup.take(),
                        &ctx.config.enforcement_facts,
                        &ctx.live_cpu_sampler,
                        SandboxCpuRetirementReason::Cancelled,
                    )
                    .await?;
                    return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox stopped after cancellation"));
                }
            }
            triggered = wait_for_watch_trigger(&mut shutdown_signal), if shutdown_signal.is_some() => {
                if triggered {
                    return handle_graceful_shutdown(
                        GracefulShutdownContext {
                            pid1: &pid1,
                            outer: &outer,
                            abort: &mut ctx.abort,
                            cancellation: &mut ctx.cancellation,
                            wall_deadline_instant,
                            shutdown_grace: ctx.config.shutdown_grace,
                            cgroup: ctx.config.cgroup.take(),
                            facts: &ctx.config.enforcement_facts,
                            live_cpu_sampler: &ctx.live_cpu_sampler,
                        },
                        &mut wait_future,
                    )
                    .await;
                }
            }
            () = &mut wall_sleep, if wall_deadline_instant.is_some() => {
                let output = reap_and_drain(
                    &pid1,
                    &outer,
                    &mut wait_future,
                    ctx.config.cgroup.take(),
                    &ctx.config.enforcement_facts,
                    &ctx.live_cpu_sampler,
                    SandboxCpuRetirementReason::TimedOut,
                )
                .await
                .ok()
                .flatten();
                let msg = format_deadline_diagnostic(
                    "local sandbox wall-clock deadline exceeded",
                    ctx.config.wall_deadline,
                    output.as_ref(),
                    &ctx.config.enforcement_facts,
                );
                return Err(classified_io_error(
                    ErrorKind::TimedOut,
                    SandboxFailureClass::WallDeadlineExceeded,
                    msg,
                ));
            }
            () = &mut silence_sleep, if silence_dur.is_some() => {
                let dur = silence_dur.expect("silence_sleep branch guarded by silence_dur.is_some()");
                let last_active = activity.last_activity_instant().max(payload_started);
                let next_deadline = last_active + dur;
                if TokioInstant::now() < next_deadline {
                    silence_sleep.as_mut().reset(next_deadline);
                } else {
                    let output = reap_and_drain(
                        &pid1,
                        &outer,
                        &mut wait_future,
                        ctx.config.cgroup.take(),
                        &ctx.config.enforcement_facts,
                        &ctx.live_cpu_sampler,
                        SandboxCpuRetirementReason::TimedOut,
                    )
                    .await
                    .ok()
                    .flatten();
                    let msg = format_deadline_diagnostic(
                        "local sandbox silence budget exceeded",
                        silence_dur,
                        output.as_ref(),
                        &ctx.config.enforcement_facts,
                    );
                    return Err(classified_io_error(
                        ErrorKind::TimedOut,
                        SandboxFailureClass::SilenceBudgetExceeded,
                        msg,
                    ));
                }
            }
            result = &mut wait_future => {
                let cgroup_arc = ctx.config.cgroup.take();
                if ctx.abort.try_recv().is_ok() {
                    retire_live_cpu_sampler(&ctx.live_cpu_sampler, SandboxCpuRetirementReason::DroppedOrAborted);
                    terminate_until_observed(&pid1).await;
                    if result.is_err() {
                        wait_outer_exit(&outer).await;
                    }
                    let _ = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await;
                    return Err(io::Error::new(ErrorKind::Interrupted, "dropped watch build stopped by supervisor"));
                }
                if is_watch_triggered(&ctx.cancellation) {
                    retire_live_cpu_sampler(&ctx.live_cpu_sampler, SandboxCpuRetirementReason::Cancelled);
                    terminate_until_observed(&pid1).await;
                    if result.is_err() {
                        wait_outer_exit(&outer).await;
                    }
                    let _ = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await;
                    return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox stopped after cancellation"));
                }
                if is_watch_triggered(&shutdown_signal) {
                    retire_live_cpu_sampler(&ctx.live_cpu_sampler, SandboxCpuRetirementReason::GracefulShutdown);
                    terminate_until_observed(&pid1).await;
                    if result.is_err() {
                        wait_outer_exit(&outer).await;
                    }
                    let _ = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await;
                    return Err(io::Error::new(ErrorKind::Interrupted, "local sandbox stopped by supervisor shutdown"));
                }
                if wall_deadline_instant.is_some_and(|d| TokioInstant::now() >= d) {
                    retire_live_cpu_sampler(&ctx.live_cpu_sampler, SandboxCpuRetirementReason::TimedOut);
                    terminate_until_observed(&pid1).await;
                    if result.is_err() {
                        wait_outer_exit(&outer).await;
                    }
                    let _ = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await;
                    return Err(classified_io_error(
                        ErrorKind::TimedOut,
                        SandboxFailureClass::WallDeadlineExceeded,
                        "local sandbox wall-clock deadline exceeded concurrently with completion",
                    ));
                }
                if let Some(dur) = silence_dur {
                    let last_active = activity.last_activity_instant().max(payload_started);
                    if TokioInstant::now() >= last_active + dur {
                        retire_live_cpu_sampler(&ctx.live_cpu_sampler, SandboxCpuRetirementReason::TimedOut);
                        terminate_until_observed(&pid1).await;
                        if result.is_err() {
                            wait_outer_exit(&outer).await;
                        }
                        let _ = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await;
                        return Err(classified_io_error(
                            ErrorKind::TimedOut,
                            SandboxFailureClass::SilenceBudgetExceeded,
                            "local sandbox silence budget exceeded concurrently with completion",
                        ));
                    }
                }
                match result {
                    Ok(output) => {
                        if let Some(cg) = cgroup_arc.as_ref()
                            && let Some(state) = ctx.live_cpu_sampler.as_ref()
                        {
                            state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).sample_once(cg);
                        }
                        let retire_reason = if output.status.success() {
                            SandboxCpuRetirementReason::Completed
                        } else {
                            SandboxCpuRetirementReason::Failed
                        };
                        retire_live_cpu_sampler(&ctx.live_cpu_sampler, retire_reason);
                        if let Some(events) = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await? {
                            if events.oom_kill_count > 0 {
                                return Err(classified_io_error(
                                    ErrorKind::Other,
                                    SandboxFailureClass::ResourceBudgetExceeded,
                                    format!(
                                        "local sandbox terminated by cgroup memory limit (oom_kill={}, oom={}, peak_bytes={:?})",
                                        events.oom_kill_count, events.oom_count, events.memory_peak_bytes
                                    ),
                                ));
                            }
                            if !output.status.success() && events.pids_max_events > 0 {
                                return Err(classified_io_error(
                                    ErrorKind::Other,
                                    SandboxFailureClass::ResourceBudgetExceeded,
                                    format!(
                                        "local sandbox hit cgroup pids limit (pids_max_events={}, status={})",
                                        events.pids_max_events, output.status
                                    ),
                                ));
                            }
                        }
                        return Ok(output);
                    }
                    Err(error) => {
                        retire_live_cpu_sampler(&ctx.live_cpu_sampler, SandboxCpuRetirementReason::Failed);
                        terminate_until_observed(&pid1).await;
                        wait_outer_exit(&outer).await;
                        let _ = finalize_cgroup(cgroup_arc, &ctx.config.enforcement_facts).await;
                        return Err(error);
                    }
                }
            }
            () = &mut sample_sleep, if sampling_active => {
                if let (Some(cg), Some(state)) = (ctx.config.cgroup.as_ref(), ctx.live_cpu_sampler.as_ref()) {
                    state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).sample_once(cg);
                }
                sample_sleep.as_mut().reset(TokioInstant::now() + sample_cadence);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Stdio;

    use tokio::process::Child;
    use tokio::process::Command;

    use super::*;

    async fn spawn_escaped_descendant(dir: &Path) -> io::Result<Child> {
        let marker = dir.join("started");
        let late = dir.join("late");
        let shell = format!(
            "setsid /bin/sh -c 'sleep 2; printf late > {}' & printf started > {}; wait",
            late.display(),
            marker.display(),
        );
        Command::new(super::super::bwrap_program())
            .args([
                "--unshare-user",
                "--unshare-pid",
                "--new-session",
                "--ro-bind",
                "/",
                "/",
                "--bind",
            ])
            .arg(dir)
            .arg(dir)
            .args(["--proc", "/proc", "--", "/bin/sh", "-c"])
            .arg(shell)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    }

    async fn wait_for_started(dir: &Path) {
        let until = tokio::time::Instant::now() + Duration::from_secs(5);
        while !dir.join("started").is_file() {
            assert!(tokio::time::Instant::now() < until, "bwrap builder never started");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[test]
    fn gated_early_exit_requires_last_inherited_reader_to_close() {
        let mut gate = WatchStartGate::new().unwrap();
        let inherited_reader = gate.reader.as_ref().unwrap().try_clone().unwrap();
        gate.close_parent_reader();
        assert!(!gate.no_readers().unwrap(), "live inherited reader was misclassified as terminal");
        drop(inherited_reader);
        assert!(gate.no_readers().unwrap(), "last reader exit did not allow terminal classification");
    }

    #[tokio::test]
    async fn killing_outer_bwrap_alone_leaves_escaped_descendant() {
        let dir = tempfile::tempdir().unwrap();
        let mut outer = spawn_escaped_descendant(dir.path()).await.unwrap();
        wait_for_started(dir.path()).await;
        outer.start_kill().unwrap();
        outer.wait().await.unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert_eq!(fs::read(dir.path().join("late")).unwrap(), b"late");
    }

    #[tokio::test]
    async fn owned_namespace_pidfd_stops_escaped_descendant_before_release() {
        let dir = tempfile::tempdir().unwrap();
        let mut outer = spawn_escaped_descendant(dir.path()).await.unwrap();
        wait_for_started(dir.path()).await;
        let outer_owner = OwnedOuterBwrap::open_for_spawned_child(outer.id().unwrap()).unwrap();
        let owner = OwnedPidNamespaceLeader::open_from_outer(&outer_owner).unwrap();
        owner.terminate_and_wait(Duration::from_secs(5)).unwrap();
        outer.wait().await.unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert!(!dir.path().join("late").exists(), "retracted sandbox published a late effect");
    }
}
