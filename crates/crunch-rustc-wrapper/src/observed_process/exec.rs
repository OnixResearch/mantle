//! Successful descendant exec observations. Polling remains resource sampling only.
use std::collections::BTreeMap;
use std::ops::Bound;
use std::os::unix::process::ExitStatusExt as _;
use std::process::ExitStatus;

use super::CHILD_TASKS_MAX;
use super::ProcessFact;
use super::Result;
use super::err;

const MAX_TRACE_EVENTS: u32 = 65_536;
const TRACE_OPTIONS: libc::c_int = libc::PTRACE_O_TRACEFORK
    | libc::PTRACE_O_TRACEVFORK
    | libc::PTRACE_O_TRACECLONE
    | libc::PTRACE_O_TRACEEXEC
    | libc::PTRACE_O_EXITKILL;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    InitialExec,
    Newborn,
    Running,
}

pub(super) struct Trace {
    root_pid: libc::pid_t,
    tracees: BTreeMap<libc::pid_t, State>,
    events: u32,
    status: Option<ExitStatus>,
}

/// Called after the existing resource/group setup, immediately before exec.
/// No stop before exec: Command::spawn must first observe its exec-error pipe close.
pub(super) fn prepare() -> std::io::Result<()> {
    // SAFETY: PTRACE_TRACEME takes no pointers and applies to this child only.
    let result = unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) };
    if result == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

impl Trace {
    pub(super) fn new(root_pid: libc::pid_t) -> Self {
        assert!(root_pid > 0);
        const { assert!(CHILD_TASKS_MAX > 0) };
        Self {
            root_pid,
            tracees: BTreeMap::from([(root_pid, State::InitialExec)]),
            events: 0,
            status: None,
        }
    }

    pub(super) fn status(&self) -> Option<ExitStatus> {
        self.status
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tracees.is_empty()
    }

    /// Every successful exec is stopped by the kernel before its first instruction.
    /// Iterate only this owner's exact tracees; never reap another supervisor's child.
    pub(super) fn drain(&mut self, observe: &mut dyn FnMut(ProcessFact) -> Result<()>) -> Result<()> {
        let mut cursor = 0;
        for _ in 0..CHILD_TASKS_MAX {
            let Some((&pid, &state)) = self.tracees.range((Bound::Excluded(cursor), Bound::Unbounded)).next() else {
                return Ok(());
            };
            cursor = pid;
            let mut status = 0;
            // SAFETY: status is writable; each PID belongs to this exact tracer.
            let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG | libc::__WALL) };
            if result == 0 {
                continue;
            }
            if result == -1 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(format!("exec-observer-wait: {error}"));
            }
            assert_eq!(result, pid);
            self.accept(pid, state, status, observe)?;
        }
        assert!(u32::try_from(self.tracees.len()).is_ok_and(|count| count <= CHILD_TASKS_MAX));
        Ok(())
    }

    fn accept(
        &mut self,
        pid: libc::pid_t,
        state: State,
        status: libc::c_int,
        observe: &mut dyn FnMut(ProcessFact) -> Result<()>,
    ) -> Result<()> {
        assert!(pid > 0);
        if self.events >= MAX_TRACE_EVENTS {
            return Err("exec-observer-event-bound".into());
        }
        self.events = self.events.saturating_add(1);
        if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
            self.tracees.remove(&pid);
            if pid == self.root_pid {
                self.status = Some(ExitStatus::from_raw(status));
            }
            return Ok(());
        }
        if !libc::WIFSTOPPED(status) {
            return Err("exec-observer-unexpected-wait-status".into());
        }
        assert!(self.events <= MAX_TRACE_EVENTS);
        self.stopped(pid, state, status, observe)
    }

    fn stopped(
        &mut self,
        pid: libc::pid_t,
        state: State,
        status: libc::c_int,
        observe: &mut dyn FnMut(ProcessFact) -> Result<()>,
    ) -> Result<()> {
        let signal = libc::WSTOPSIG(status);
        let event = status >> 16;
        if state == State::InitialExec {
            if signal != libc::SIGTRAP || event != 0 {
                return Err("exec-observer-initial-stop".into());
            }
            request(libc::PTRACE_SETOPTIONS, pid, libc::c_long::from(TRACE_OPTIONS))?;
            observe(process(pid)?)?;
        } else if event == libc::PTRACE_EVENT_EXEC {
            let previous_tid = event_pid(pid)?;
            if previous_tid != pid {
                self.tracees.remove(&previous_tid);
            }
            observe(process(pid)?)?;
        } else if matches!(event, libc::PTRACE_EVENT_FORK | libc::PTRACE_EVENT_VFORK | libc::PTRACE_EVENT_CLONE) {
            self.forked(pid)?;
        } else if event == 0 && signal == libc::SIGSTOP {
            if state != State::Newborn {
                // Preserve a real stop. The normal owned deadline/kill still applies.
                return Ok(());
            }
        } else if event == 0 {
            return request(libc::PTRACE_CONT, pid, libc::c_long::from(signal));
        } else {
            return Err("exec-observer-unknown-event".into());
        }
        assert!(pid > 0);
        assert!(self.tracees.contains_key(&pid));
        self.tracees.insert(pid, State::Running);
        request(libc::PTRACE_CONT, pid, 0)
    }

    fn forked(&mut self, pid: libc::pid_t) -> Result<()> {
        let child = event_pid(pid)?;
        if child <= 0 || u32::try_from(self.tracees.len()).map_err(err)? >= CHILD_TASKS_MAX {
            return Err("exec-observer-task-bound".into());
        }
        assert!(child > 0);
        assert!(!self.tracees.contains_key(&child));
        self.tracees.insert(child, State::Newborn);
        Ok(())
    }

    /// Reap every task of the owned process group after it was killed, including
    /// traced threads whose creation event was never drained. A traced thread-group
    /// leader cannot be reaped until each of its traced threads has been, so an
    /// untracked thread would otherwise keep the leader unreapable forever.
    pub(super) fn reap_killed_group(&mut self) {
        assert!(self.root_pid > 0);
        assert!(u32::try_from(self.tracees.len()).is_ok_and(|count| count <= CHILD_TASKS_MAX));
        for _ in 0..CHILD_TASKS_MAX.saturating_mul(4) {
            let mut status = 0;
            // SAFETY: status is writable; -root_pid selects only the owned process
            // group, whose leader is this tracer's directly owned command.
            let pid = unsafe { libc::waitpid(-self.root_pid, &mut status, libc::WNOHANG | libc::__WALL) };
            if pid <= 0 {
                return;
            }
            if libc::WIFSTOPPED(status) {
                // A stop reported before the kill; the pending SIGKILL still ends the
                // stopped tracee, whose exit a later wait reaps.
                continue;
            }
            self.tracees.remove(&pid);
            if pid == self.root_pid {
                self.status = Some(ExitStatus::from_raw(status));
            }
        }
    }
}

impl Drop for Trace {
    fn drop(&mut self) {
        assert!(self.root_pid > 0);
        assert!(u32::try_from(self.tracees.len()).is_ok_and(|count| count <= CHILD_TASKS_MAX));
        for &pid in self.tracees.keys() {
            // SAFETY: unreaped tracees retain their exact PID, preventing reuse.
            unsafe { libc::kill(pid, libc::SIGKILL) };
        }
        // Kill members whose creation event was never drained, then reap them too.
        let _ = super::signal_group(super::GroupSignal {
            pid: self.root_pid,
            signal: libc::SIGKILL,
        });
        let started = std::time::Instant::now();
        for _ in 0..super::MAX_TEARDOWN_POLLS {
            self.reap_killed_group();
            if self.tracees.is_empty() || started.elapsed() >= std::time::Duration::from_millis(super::TEARDOWN_MS) {
                break;
            }
            let mut cursor = 0;
            for _ in 0..CHILD_TASKS_MAX {
                let Some((&pid, _)) = self.tracees.range((Bound::Excluded(cursor), Bound::Unbounded)).next() else {
                    break;
                };
                cursor = pid;
                let mut status = 0;
                // SAFETY: bounded nonblocking reap of this owner's tracee only.
                let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG | libc::__WALL) };
                if result == pid {
                    if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
                        self.tracees.remove(&pid);
                    } else if libc::WIFSTOPPED(status) {
                        let _ = request(libc::PTRACE_CONT, pid, libc::c_long::from(libc::SIGKILL));
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(super::POLL_MS));
        }
    }
}

fn event_pid(pid: libc::pid_t) -> Result<libc::pid_t> {
    let mut value: libc::c_ulong = 0;
    // SAFETY: the kernel writes one unsigned long for this known ptrace event.
    if unsafe { libc::ptrace(libc::PTRACE_GETEVENTMSG, pid, 0, &mut value) } == -1 {
        return Err(format!("exec-observer-event: {}", std::io::Error::last_os_error()));
    }
    let result = libc::pid_t::try_from(value).map_err(err)?;
    assert!(pid > 0);
    assert!(result > 0);
    Ok(result)
}

fn request(operation: libc::c_uint, pid: libc::pid_t, data: libc::c_long) -> Result<()> {
    assert!(pid > 0);
    assert!(matches!(operation, libc::PTRACE_SETOPTIONS | libc::PTRACE_CONT));
    // SAFETY: these requests use integer arguments, never dereference data.
    if unsafe { libc::ptrace(operation, pid, 0, data) } == -1 {
        let error = std::io::Error::last_os_error();
        // A concurrently delivered SIGKILL can remove a stopped tracee.
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(format!("exec-observer-control: {error}"));
        }
    }
    Ok(())
}

fn process(pid: libc::pid_t) -> Result<ProcessFact> {
    assert!(pid > 0);
    let root = std::path::PathBuf::from(format!("/proc/{pid}"));
    let command = super::read_process_bytes(&root.join("cmdline"))?.ok_or("exec-observer-command-unavailable")?;
    let executable = std::fs::read_link(root.join("exe")).map_err(err)?;
    assert!(executable.is_absolute());
    Ok(ProcessFact {
        pid: u32::try_from(pid).map_err(err)?,
        command: super::Digest::from_slice(&command),
        executable: Some(executable),
    })
}
