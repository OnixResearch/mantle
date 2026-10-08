//! Proposed Mantle-local watch cancellation hook for an already-owned sandbox.
//!
//! Opening a pidfd for the *inner namespace PID1*, not the outer bwrap PID,
//! binds the process identity before requesting teardown. A discovery deadline
//! is diagnostic; unresolved ownership remains gated or quarantined.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nix::libc;
use tokio::process::{Child, Command};
use tokio::sync::{oneshot, watch};
use tokio::time::{sleep, Instant as TokioInstant};

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

/// The last watch owner restores sandbox-created 0000 directories before
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
        if let Some(dir) = &self.dir {
            if let Err(error) = restore_directory_access(dir.path()) {
                tracing::warn!(?error, sandbox_path = %dir.path().display(), "watch sandbox cleanup access not restored");
            }
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

/// Admission and workspace cleanup cannot outlive this spawned sandbox.
/// The BuildService caller may be aborted while teardown is in progress.
pub(crate) struct WatchLease {
    _permit: tokio::sync::OwnedSemaphorePermit,
    _workspace: Arc<WatchWorkspace>,
}

impl WatchLease {
    pub(crate) fn new(permit: tokio::sync::OwnedSemaphorePermit, workspace: Arc<WatchWorkspace>) -> Self {
        Self { _permit: permit, _workspace: workspace }
    }
}

/// An outer bwrap process identity pinned *before* inspecting `/proc` children.
pub(crate) struct OwnedOuterBwrap {
    pid: u32,
    pidfd: OwnedFd,
}

/// Reject a watch build *before spawning* if the kernel cannot create pidfds.
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
        Ok(Self { pid, pidfd: unsafe { OwnedFd::from_raw_fd(raw as i32) } })
    }

    fn is_exited(&self) -> io::Result<bool> {
        let mut watched = libc::pollfd { fd: self.pidfd.as_raw_fd(), events: libc::POLLIN, revents: 0 };
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
    /// bwrap. A missing child is retryable while that outer process is alive.
    pub(crate) fn open_from_outer(outer: &OwnedOuterBwrap) -> io::Result<Self> {
        if outer.is_exited()? {
            return Err(io::Error::new(ErrorKind::WouldBlock, "outer bwrap exited before PID1 ownership"));
        }
        let outer_pid = outer.pid;
        let children = fs::read_to_string(format!("/proc/{outer_pid}/task/{outer_pid}/children"))?;
        for child in children.split_whitespace().take(32) {
            let child_pid = child.parse::<u32>().map_err(|_| io::Error::other("invalid bwrap child pid"))?;
            if !owned_namespace_leader(outer_pid, child_pid)? {
                continue;
            }
            // SAFETY: pidfd_open is a Linux syscall returning a new owned file
            // descriptor on success. No process identity is recovered from a
            // stale numeric PID after this point.
            let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, child_pid, 0) };
            if raw < 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: a successful pidfd_open returns a unique owned fd.
            let pidfd = unsafe { OwnedFd::from_raw_fd(raw as i32) };
            // Close an fd opened during parentage/PID namespace turnover rather
            // than misidentifying an unrelated process.
            if !outer.is_exited()? && owned_namespace_leader(outer_pid, child_pid)? {
                return Ok(Self { pidfd });
            }
            return Err(io::Error::new(ErrorKind::WouldBlock, "namespace leader changed during ownership check"));
        }
        Err(io::Error::new(ErrorKind::WouldBlock, "outer bwrap has no observed namespace PID1"))
    }

    /// SIGKILL the owned PID namespace leader and observe its actual exit.
    /// Linux kills all members of that PID namespace when its init exits,
    /// including children that escaped the original process group via setsid.
    pub(crate) fn terminate_and_wait(&self, deadline: Duration) -> io::Result<()> {
        // SAFETY: pidfd_send_signal targets this previously opened owned fd,
        // not a reused numeric PID. A zero siginfo pointer is supported.
        let sent = unsafe {
            libc::syscall(libc::SYS_pidfd_send_signal, self.pidfd.as_raw_fd(), libc::SIGKILL, 0, 0)
        };
        if sent < 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err(io::Error::last_os_error());
        }
        let until = Instant::now() + deadline;
        loop {
            let mut watched = libc::pollfd { fd: self.pidfd.as_raw_fd(), events: libc::POLLIN, revents: 0 };
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

fn owned_namespace_leader(outer_pid: u32, child_pid: u32) -> io::Result<bool> {
    let status = fs::read_to_string(format!("/proc/{child_pid}/status"))?;
    let parent = status.lines().find_map(|line| line.strip_prefix("PPid:")).and_then(|text| text.trim().parse::<u32>().ok());
    let innermost_pid = status
        .lines()
        .find_map(|line| line.strip_prefix("NSpid:"))
        .and_then(|text| text.split_whitespace().next_back())
        .and_then(|text| text.parse::<u32>().ok());
    Ok(parent == Some(outer_pid) && innermost_pid == Some(1))
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

    pub(crate) fn configure(&self, command: &mut Command) {
        let fd = self.reader.as_ref().expect("gate reader is configured before spawn").as_raw_fd();
        command.arg("--block-fd").arg(fd.to_string());
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
    let mut watched = libc::pollfd { fd: writer.as_raw_fd(), events: libc::POLLOUT, revents: 0 };
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

/// The supervisor, not the awaiting BuildService future, owns the child and
/// startup gate. Dropping the future requests cancellation without closing
/// the gate or losing the PID1/outer reaping obligation.
pub(crate) async fn run_child(
    mut command: Command,
    program: &OsString,
    cancellation: watch::Receiver<bool>,
    mut gate: WatchStartGate,
    lease: WatchLease,
    #[cfg(test)] fault: Option<WatchFault>,
) -> io::Result<std::process::Output> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    // Make the emergency writer while no child exists; a failed dup must not
    // strand a spawned sandbox. A supervisor panic cannot create gate EOF.
    let panic_writer = gate.writer.try_clone()?;
    let child = command.spawn().map_err(|error| super::annotate_bwrap_spawn_error(program, error))?;
    gate.close_parent_reader();
    let lease = Arc::new(lease);
    let identities = Arc::new(Mutex::new(SupervisorIdentities::default()));
    let (abort_tx, abort_rx) = oneshot::channel();
    let worker = tokio::spawn(supervise_child(
        child, gate, cancellation, abort_rx, Arc::clone(&lease), Arc::clone(&identities), #[cfg(test)] fault,
    ));
    // This watchdog, not the abortable caller, owns the emergency writer and
    // admission lease until even a panicking sandbox supervisor is resolved.
    let watchdog = tokio::spawn(async move {
        match worker.await {
            Ok(result) => result,
            Err(error) => {
                let (pid1, outer) = {
                    let owned = identities.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    (owned.pid1.clone(), owned.outer.clone())
                };
                if let (Some(pid1), Some(outer)) = (pid1, outer.as_ref()) {
                    terminate_until_observed(&pid1).await;
                    wait_outer_exit(outer).await;
                    return Err(io::Error::other(format!("watch supervisor failed after owned teardown: {error}")));
                }
                if let Some(outer) = outer.as_ref() {
                    let until = TokioInstant::now() + PID1_DISCOVERY_REPORT_DEADLINE;
                    loop {
                        if let Ok(pid1) = OwnedPidNamespaceLeader::open_from_outer(outer) {
                            terminate_until_observed(&Arc::new(pid1)).await;
                            wait_outer_exit(outer).await;
                            return Err(io::Error::other(format!("watch supervisor failed; recovered PID1 teardown: {error}")));
                        }
                        if outer.is_exited().unwrap_or(false) {
                            if writer_no_readers(&panic_writer).unwrap_or(false) {
                                return Err(io::Error::other(format!("watch supervisor failed after terminal bwrap exit: {error}")));
                            }
                            break;
                        }
                        if TokioInstant::now() >= until {
                            break;
                        }
                        sleep(PID1_DISCOVERY_POLL).await;
                    }
                } else if writer_no_readers(&panic_writer).unwrap_or(false) {
                    return Err(io::Error::other(format!("watch supervisor failed without an inherited gate reader: {error}")));
                }
                tokio::spawn(async move {
                    loop {
                        let outer_exited = outer.as_ref().is_none_or(|outer| outer.is_exited().unwrap_or(false));
                        if outer_exited && writer_no_readers(&panic_writer).unwrap_or(false) {
                            break;
                        }
                        sleep(PID1_DISCOVERY_POLL).await;
                    }
                    drop(lease);
                });
                Err(io::Error::new(
                    ErrorKind::Other,
                    UnobservedWatchSandbox("supervisor failed before owned PID1 teardown"),
                ))
            }
        }
    });
    let mut abort_guard = AbortOnDrop(Some(abort_tx));
    let result = watchdog.await.map_err(|error| io::Error::new(
        ErrorKind::Other,
        UnobservedWatchSandbox(if error.is_panic() { "watchdog panicked" } else { "watchdog stopped" }),
    ))?;
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
    reason: &'static str,
) -> io::Result<std::process::Output> {
    let readers_closed = gate.no_readers().unwrap_or(false);
    let outer_exited = if let Some(outer) = &outer {
        outer.is_exited().unwrap_or(false)
    } else {
        child.try_wait().ok().flatten().is_some()
    };
    if readers_closed && outer_exited {
        let output = child.wait_with_output().await?;
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
                let _ = child.wait_with_output().await;
                break;
            }
            sleep(PID1_DISCOVERY_POLL).await;
        }
        drop(lease);
    });
    Err(io::Error::new(ErrorKind::Other, UnobservedWatchSandbox(reason)))
}

async fn supervise_child(
    child: Child,
    gate: WatchStartGate,
    mut cancellation: watch::Receiver<bool>,
    mut abort: oneshot::Receiver<()>,
    lease: Arc<WatchLease>,
    identities: Arc<Mutex<SupervisorIdentities>>,
    #[cfg(test)] fault: Option<WatchFault>,
) -> io::Result<std::process::Output> {
    let Some(outer_pid) = child.id() else {
        return classify_unbound_child(child, gate, lease, None, "spawned child has no PID").await;
    };
    let outer_deadline = TokioInstant::now() + PID1_DISCOVERY_REPORT_DEADLINE;
    let outer = loop {
        match OwnedOuterBwrap::open_for_spawned_child(outer_pid) {
            Ok(owner) => {
                let owner = Arc::new(owner);
                identities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).outer = Some(Arc::clone(&owner));
                break owner;
            }
            Err(error) => {
                if TokioInstant::now() >= outer_deadline {
                    tracing::error!(?error, "watch outer pidfd unavailable after spawn; classifying gated child");
                    return classify_unbound_child(child, gate, lease, None, "outer pidfd unavailable").await;
                }
                sleep(PID1_DISCOVERY_POLL).await;
            }
        }
    };
    let mut reported_discovery_failure = false;
    #[cfg(test)]
    let delayed_until = matches!(fault, Some(WatchFault::DelayPid1Discovery))
        .then(|| TokioInstant::now() + Duration::from_secs(6));
    let report_after = TokioInstant::now() + PID1_DISCOVERY_REPORT_DEADLINE;
    let pid1 = loop {
        #[cfg(test)]
        if delayed_until.is_some_and(|deadline| TokioInstant::now() < deadline) {
            if !reported_discovery_failure && TokioInstant::now() >= report_after {
                tracing::error!("watch namespace PID1 discovery deadline exceeded; start gate and reservation remain held");
                reported_discovery_failure = true;
            }
            sleep(PID1_DISCOVERY_POLL).await;
            continue;
        }
        match OwnedPidNamespaceLeader::open_from_outer(&outer) {
            Ok(owner) => {
                let owner = Arc::new(owner);
                identities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).pid1 = Some(Arc::clone(&owner));
                break owner;
            }
            Err(error) => {
                if outer.is_exited().unwrap_or(false) {
                    return classify_unbound_child(child, gate, lease, Some(outer), "outer exited before namespace PID1").await;
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
    if matches!(fault, Some(WatchFault::PanicAfterPid1)) {
        panic!("injected supervisor failure after owned PID1 before payload release");
    }
    #[cfg(test)]
    if matches!(fault, Some(WatchFault::PanicAfterAbort)) {
        fs::write(lease._workspace.path().join("watch-pid1-owned"), b"owned")
            .expect("fault fixture must publish post-PID1 readiness");
        let _ = (&mut abort).await;
        panic!("injected supervisor failure after caller abort before payload release");
    }

    let wait_future = child.wait_with_output();
    tokio::pin!(wait_future);
    if *cancellation.borrow() || cancellation.has_changed().is_err() || abort.try_recv().is_ok() {
        terminate_until_observed(&pid1).await;
        match wait_future.await {
            Ok(_) => {}
            Err(error) => {
                wait_outer_exit(&outer).await;
                return Err(error);
            }
        }
        return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox stopped before builder start"));
    }
    if let Err(error) = gate.release() {
        terminate_until_observed(&pid1).await;
        if wait_future.await.is_err() {
            wait_outer_exit(&outer).await;
        }
        return Err(error);
    }
    loop {
        tokio::select! {
            biased;
            _ = &mut abort => {
                terminate_until_observed(&pid1).await;
                match wait_future.await {
                    Ok(_) => {}
                    Err(error) => {
                        wait_outer_exit(&outer).await;
                        return Err(error);
                    }
                }
                return Err(io::Error::new(ErrorKind::Interrupted, "dropped watch build stopped by supervisor"));
            }
            change = cancellation.changed() => {
                if change.is_err() || *cancellation.borrow_and_update() {
                    terminate_until_observed(&pid1).await;
                    match wait_future.await {
                        Ok(_) => {}
                        Err(error) => {
                            wait_outer_exit(&outer).await;
                            return Err(error);
                        }
                    }
                    return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox stopped after cancellation"));
                }
            }
            result = &mut wait_future => {
                if *cancellation.borrow() || cancellation.has_changed().is_err() || abort.try_recv().is_ok() {
                    terminate_until_observed(&pid1).await;
                    if result.is_err() {
                        wait_outer_exit(&outer).await;
                    }
                    return Err(io::Error::new(ErrorKind::Interrupted, "watch sandbox stopped after cancellation"));
                }
                match result {
                    Ok(output) => return Ok(output),
                    Err(error) => {
                        terminate_until_observed(&pid1).await;
                        wait_outer_exit(&outer).await;
                        return Err(error);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Stdio;
    use tokio::process::Child;
    use tokio::process::Command;

    async fn spawn_escaped_descendant(dir: &Path) -> io::Result<Child> {
        let marker = dir.join("started");
        let late = dir.join("late");
        let shell = format!(
            "setsid /bin/sh -c 'sleep 2; printf late > {}' & printf started > {}; wait",
            late.display(), marker.display(),
        );
        Command::new(super::super::bwrap_program())
            .args(["--unshare-user", "--unshare-pid", "--new-session", "--ro-bind", "/", "/", "--bind"])
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
