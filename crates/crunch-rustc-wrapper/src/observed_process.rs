//! Bounded Linux process observation, separate from execution/cache authority.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use mantle_rust_differential_core::Digest;
use serde::Deserialize;
use serde::Serialize;

mod exec;
type Result<T, E = String> = std::result::Result<T, E>;
fn err(value: impl std::fmt::Display) -> String {
    value.to_string()
}
pub const STREAM_BYTES: u64 = 8_388_608;
pub const SCRATCH_BYTES: u64 = 2_147_483_648;
pub const FILES_MAX: u32 = 200_000;
pub const PROCESS_MAX: usize = 64;
pub const BUILD_MS: u64 = 120_000;
pub const PROGRAM_MS: u64 = 5_000;
pub const PROBE_MS: u64 = 10_000;
pub const TOTAL_MS: u64 = 7_200_000;
pub const TEARDOWN_MS: u64 = 5_000;
const POLL_MS: u64 = 2;
const DISK_CHECK_MS: u64 = 100;
const TERMINATE_GRACE_MS: u64 = 100;
// Each unsuccessful monitoring pass sleeps before the next observation. Include
// the initial pass, including when the caller's deadline is already exhausted.
const MAX_MONITOR_POLLS: u64 = TOTAL_MS.saturating_div(POLL_MS).saturating_add(1);
const MAX_PROCESS_FACTS: u64 = MAX_MONITOR_POLLS.saturating_mul(PROCESS_MAX as u64);
// Successful nonblocking reaps need not sleep, but the killed group has at most
// PROCESS_MAX members. All remaining passes sleep or finish at the deadline.
const MAX_TEARDOWN_POLLS: u64 =
    TEARDOWN_MS.saturating_div(POLL_MS).saturating_add(PROCESS_MAX as u64).saturating_add(1);
const ADDRESS_SPACE_BYTES: u64 = 8_589_934_592;
const CPU_SECONDS: u64 = 120;
const FILE_BYTES: u64 = 134_217_728;
const OPEN_FILES: u64 = 256;
// Watchdog thresholds concern this group, not unrelated tasks of its real UID.
// RLIMIT_NPROC is inherited: lowering it counts every task of that UID and can
// prevent even rustc's signal-handler thread from starting. These sampled
// thresholds are not a kernel-enforced pids controller.
const CHILD_TASKS_MAX: u32 = 128;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;
const PERMISSION_BITS: u32 = 0o777;
const STAT_GROUP_INDEX: usize = 2;
const STAT_THREADS_INDEX: usize = 17;
// Linux process IDs fit pid_t; the remaining entries are kernel-owned procfs
// nodes. This ceiling exceeds the entire signed PID namespace, not just the
// fixture's simultaneous process allowance.
const MAX_PROC_ENTRIES: u64 = (i32::MAX as u64).saturating_add(FILES_MAX as u64);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capture {
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub cwd: PathBuf,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub monotonic_origin_ns: u64,
    pub elapsed_ns: u64,
    pub last_reap_ns: u64,
    pub complete: bool,
    pub teardown_complete: bool,
    pub failure: Option<String>,
    pub trigger: Option<Trigger>,
    pub processes: Vec<ProcessFact>,
    pub resources: mantle_rust_differential_core::waits::ResourceReport,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessFact {
    pub pid: u32,
    pub command: Digest,
    pub executable: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trigger {
    pub pid: u32,
    pub executable: PathBuf,
    pub command: Digest,
    pub observed_ns: u64,
    pub signal: i32,
    pub delivered: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fault {
    Crash,
    Cancel,
}
pub struct Injection {
    pub fault: Fault,
    pub rustc: PathBuf,
    pub manifest: PathBuf,
}
pub fn ns(start: Instant) -> u64 {
    // Saturation is already beyond every admitted supervision deadline.
    let elapsed = start.elapsed();
    elapsed.as_secs().saturating_mul(1_000_000_000).saturating_add(u64::from(elapsed.subsec_nanos()))
}
fn clock_offset(origin: u64) -> u64 {
    crate::waits::monotonic_ns().unwrap_or(origin).saturating_sub(origin)
}
pub fn private_dir(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err("private-root-not-absolute".into());
    }
    let mut ancestor = PathBuf::from("/");
    for part in path.parent().ok_or("root-parent")?.components() {
        if let Component::Normal(name) = part {
            ancestor.push(name);
            let m = fs::symlink_metadata(&ancestor).map_err(err)?;
            if !m.is_dir() || m.file_type().is_symlink() {
                return Err("symlink-or-nondirectory-root-ancestor".into());
            }
        }
    }
    fs::DirBuilder::new().mode(PRIVATE_DIRECTORY_MODE).create(path).map_err(err)?;
    let metadata = fs::symlink_metadata(path).map_err(err)?;
    if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & PERMISSION_BITS != PRIVATE_DIRECTORY_MODE {
        return Err("private-root-owner-mode".into());
    }
    assert!(metadata.is_dir());
    assert!(!metadata.file_type().is_symlink());
    Ok(())
}
pub fn tree_bytes(root: &Path, files: &mut u32, total: &mut u64) -> Result<()> {
    // Keep the recursive traversal order without consuming the call stack.
    // Every pushed child directory has already been charged to FILES_MAX.
    let mut directories = vec![fs::read_dir(root).map_err(err)?];
    // An entry and each finished directory consume at most two passes per file.
    for _ in 0..FILES_MAX.saturating_mul(2).saturating_add(2) {
        let Some(directory) = directories.last_mut() else {
            return Ok(());
        };
        let Some(entry) = directory.next() else {
            directories.pop();
            continue;
        };
        let e = entry.map_err(err)?;
        *files = files.checked_add(1).ok_or("file-count-overflow")?;
        if *files > FILES_MAX {
            return Err("scratch-file-bound".into());
        }
        // Cargo removes and renames temporary members during observation. A
        // vanished member no longer contributes; a missing root or any other
        // I/O error still fails the sample.
        let metadata = match e.metadata() {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(err(error)),
        };
        if metadata.is_dir() {
            match fs::read_dir(e.path()) {
                Ok(directory) => directories.push(directory),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(err(error)),
            }
        } else {
            *total = total.checked_add(metadata.len()).ok_or("scratch-overflow")?;
        }
        if *total > SCRATCH_BYTES {
            return Err("scratch-byte-bound".into());
        }
        assert!(*files <= FILES_MAX);
        assert!(*total <= SCRATCH_BYTES);
        assert!(u64::try_from(directories.len()).map_err(err)? <= u64::from(*files).saturating_add(1));
    }
    Err("scratch-file-bound".into())
}
fn group(pid: i32) -> Result<Vec<ProcessFact>> {
    assert!(pid > 0);
    let mut found = vec![];
    let mut tasks = 0;
    for (entry_count, entry) in fs::read_dir("/proc").map_err(err)?.enumerate() {
        if u64::try_from(entry_count).map_err(err)? >= MAX_PROC_ENTRIES {
            return Err("proc-entry-bound".into());
        }
        let entry = entry.map_err(err)?;
        let Ok(id) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Some(stat) = read_process_bytes(&entry.path().join("stat"))? else {
            continue;
        };
        let Ok(stat) = String::from_utf8(stat) else {
            continue;
        };
        let Some((_, tail)) = stat.rsplit_once(") ") else {
            continue;
        };
        if tail.split_whitespace().nth(STAT_GROUP_INDEX).and_then(|value| value.parse::<i32>().ok()) != Some(pid) {
            continue;
        }
        if found.len() >= PROCESS_MAX {
            return Err("process-count-bound".into());
        }
        let threads = tail
            .split_whitespace()
            .nth(STAT_THREADS_INDEX)
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or("process-thread-count")?;
        tasks = admit_process_tasks(ProcessTasks {
            total: tasks,
            observed: threads,
        })?;
        let command = read_process_bytes(&entry.path().join("cmdline"))?.unwrap_or_default();
        found.push(ProcessFact {
            pid: id,
            command: Digest::from_slice(&command),
            executable: fs::read_link(entry.path().join("exe")).ok(),
        });
        assert!(found.len() <= PROCESS_MAX);
    }
    Ok(found)
}
struct ProcessTasks {
    total: u32,
    observed: u32,
}

fn admit_process_tasks(counts: ProcessTasks) -> Result<u32> {
    let ProcessTasks { total, observed: tasks } = counts;
    assert!(total <= CHILD_TASKS_MAX);
    if tasks == 0 {
        return Err("process-thread-count".into());
    }
    let next = total.checked_add(tasks).ok_or("process-thread-count-overflow")?;
    if next > CHILD_TASKS_MAX {
        return Err("process-thread-count-bound".into());
    }
    assert!(next > total);
    assert!(next <= CHILD_TASKS_MAX);
    Ok(next)
}

fn read_process_bytes(path: &Path) -> Result<Option<Vec<u8>>> {
    let Ok(file) = File::open(path) else {
        return Ok(None);
    };
    let mut bytes = vec![];
    let read_bound_bytes = ADDRESS_SPACE_BYTES.checked_add(1).ok_or("process-metadata-bound-overflow")?;
    if file.take(read_bound_bytes).read_to_end(&mut bytes).is_err() {
        return Ok(None);
    }
    if u64::try_from(bytes.len()).map_err(err)? > ADDRESS_SPACE_BYTES {
        return Err("process-metadata-bound".into());
    }
    Ok(Some(bytes))
}
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if let Ok(pid) = i32::try_from(self.0.id()) {
            let _ = signal_group(GroupSignal {
                pid,
                signal: libc::SIGKILL,
            });
        }
        wait_bounded(&mut self.0);
    }
}

/// Reap the owned child without blocking teardown forever: a traced leader is
/// reapable only after its tracer reaped every traced thread.
fn wait_bounded(child: &mut Child) {
    assert!(child.id() > 0);
    const { assert!(POLL_MS > 0) };
    let started = Instant::now();
    for _ in 0..MAX_TEARDOWN_POLLS {
        if !matches!(child.try_wait(), Ok(None)) || started.elapsed() >= Duration::from_millis(TEARDOWN_MS) {
            return;
        }
        thread::sleep(Duration::from_millis(POLL_MS));
    }
}
struct GroupSignal {
    pid: i32,
    signal: i32,
}

fn signal_group(request: GroupSignal) -> Result<()> {
    let GroupSignal { pid, signal } = request;
    assert!(pid > 0);
    assert!(matches!(signal, libc::SIGTERM | libc::SIGKILL));
    if unsafe { libc::kill(-pid, signal) } != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
        return Err("group-signal".into());
    }
    Ok(())
}
/// Diagnostic callbacks occur at actual process boundaries, never reconstructed
/// from a completed capture. Observers cannot alter execution or admission.
pub enum ProcessEvent {
    Spawned(u32),
    Exited {
        code: Option<i32>,
        signal: Option<i32>,
        timeout: bool,
    },
    Reaped,
}

/// Bounds and owned paths for one supervised command. Process/task limits
/// apply only to the command's process group, never all tasks of its real UID.
pub struct Supervision<'a> {
    pub root: &'a Path,
    pub scratch: &'a Path,
    pub timeout_ms: u64,
    pub total_start: Instant,
    pub injection: Option<Injection>,
    pub stream_bound: u64,
}

pub fn supervise(command: Command, options: Supervision<'_>) -> Result<Capture> {
    supervise_observed(command, options, &mut |_| {})
}

struct CommandDescription {
    argv: Vec<String>,
    environment: BTreeMap<String, String>,
    cwd: PathBuf,
}

impl CommandDescription {
    fn read(command: &Command) -> Result<Self> {
        // Charge source bytes and a terminator per field before copying. This
        // also bounds empty arguments; lossy UTF-8 expands at most threefold.
        let mut remaining_bytes = ADDRESS_SPACE_BYTES;
        let cwd = command.get_current_dir().ok_or("command-cwd")?;
        charge_text(cwd.as_os_str(), &mut remaining_bytes)?;
        let argv: Vec<String> = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|value| capture_text(value, &mut remaining_bytes))
            .collect::<Result<_>>()?;
        let environment = command_environment(command, &mut remaining_bytes)?;
        assert!(!argv.is_empty());
        assert!(remaining_bytes <= ADDRESS_SPACE_BYTES);
        Ok(Self {
            argv,
            environment,
            cwd: cwd.to_path_buf(),
        })
    }
}

fn command_environment(command: &Command, remaining_bytes: &mut u64) -> Result<BTreeMap<String, String>> {
    command
        .get_envs()
        .filter_map(|(key, value)| value.map(|value| (key, value)))
        .map(|(key, value)| Ok((capture_text(key, remaining_bytes)?, capture_text(value, remaining_bytes)?)))
        .collect()
}

fn charge_text(value: &std::ffi::OsStr, remaining_bytes: &mut u64) -> Result<()> {
    let source_bytes = value.len().checked_add(1).ok_or("command-metadata-overflow")?;
    *remaining_bytes = remaining_bytes
        .checked_sub(u64::try_from(source_bytes).map_err(err)?)
        .ok_or("command-metadata-bound")?;
    Ok(())
}

fn capture_text(value: &std::ffi::OsStr, remaining_bytes: &mut u64) -> Result<String> {
    charge_text(value, remaining_bytes)?;
    Ok(value.to_string_lossy().into_owned())
}

struct OutputFiles {
    stdout: PathBuf,
    stderr: PathBuf,
    bound_bytes: u64,
}

impl OutputFiles {
    fn attach(command: &mut Command, root: &Path, bound_bytes: u64) -> Result<Self> {
        let files = Self {
            stdout: root.join("stdout.raw"),
            stderr: root.join("stderr.raw"),
            bound_bytes,
        };
        command
            .stdin(Stdio::null())
            .stdout(Stdio::from(open_output(&files.stdout)?))
            .stderr(Stdio::from(open_output(&files.stderr)?));
        Ok(files)
    }

    fn exceeded(&self) -> Result<bool> {
        Ok(fs::metadata(&self.stdout).map_err(err)?.len() > self.bound_bytes
            || fs::metadata(&self.stderr).map_err(err)?.len() > self.bound_bytes)
    }

    fn prefix(&self, path: &Path) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        File::open(path)
            .map_err(err)?
            .take(self.bound_bytes.min(FILE_BYTES))
            .read_to_end(&mut bytes)
            .map_err(err)?;
        assert!(u64::try_from(bytes.len()).map_err(err)? <= self.bound_bytes);
        assert!(u64::try_from(bytes.len()).map_err(err)? <= FILE_BYTES);
        Ok(bytes)
    }
}

fn open_output(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(PRIVATE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(err)
}

fn configure_child(command: &mut Command) {
    unsafe {
        command.pre_exec(child_limits);
    }
}

fn child_limits() -> std::io::Result<()> {
    establish_group()?;
    for (kind, value) in [
        (libc::RLIMIT_AS, ADDRESS_SPACE_BYTES),
        (libc::RLIMIT_CPU, CPU_SECONDS),
        (libc::RLIMIT_FSIZE, FILE_BYTES),
        (libc::RLIMIT_NOFILE, OPEN_FILES),
        (libc::RLIMIT_CORE, 0),
    ] {
        if unsafe { libc::setrlimit(kind, &resource_limit(value)) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

fn establish_group() -> std::io::Result<()> {
    if unsafe { libc::setpgid(0, 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn resource_limit(value: libc::rlim_t) -> libc::rlimit {
    libc::rlimit {
        rlim_cur: value,
        rlim_max: value,
    }
}

struct Monitor<'a> {
    scratch: &'a Path,
    timeout_ms: u64,
    total_start: Instant,
    injection: Option<Injection>,
    started: Instant,
    disk_check: Instant,
    origin_ns: u64,
    trigger: Option<Trigger>,
    failure: Option<String>,
    processes: BTreeMap<u32, Vec<ProcessFact>>,
    process_fact_count: u64,
    sampler: crate::waits::ProcessTreeSampler,
}

impl Monitor<'_> {
    fn deadline_expired(&self) -> bool {
        self.started.elapsed() >= Duration::from_millis(self.timeout_ms)
            || self.total_start.elapsed() >= Duration::from_millis(TOTAL_MS)
    }

    fn observe(&mut self, pid: i32) -> Result<()> {
        let observed = group(pid)?;
        assert!(observed.len() <= PROCESS_MAX);
        if self.trigger.is_none()
            && let Some(injection) = &self.injection
        {
            self.trigger = inject_compiler(injection, &observed, pid, self.origin_ns)?;
        }
        self.sampler.sample(clock_offset(self.origin_ns));
        for process in observed {
            self.record_process(process)?;
        }
        assert!(self.process_fact_count <= MAX_PROCESS_FACTS);
        Ok(())
    }

    fn observe_exec(&mut self, process: ProcessFact, group_pid: i32) -> Result<()> {
        assert!(process.executable.is_some());
        assert!(process.pid > 0);
        if self.trigger.is_none()
            && let Some(injection) = &self.injection
        {
            self.trigger = inject_compiler(injection, std::slice::from_ref(&process), group_pid, self.origin_ns)?;
        }
        self.record_process(process)
    }

    fn record_process(&mut self, process: ProcessFact) -> Result<()> {
        assert!(process.pid > 0);
        assert!(self.process_fact_count <= MAX_PROCESS_FACTS);
        let history = self.processes.entry(process.pid).or_default();
        if history
            .last()
            .is_some_and(|previous| previous.command == process.command && previous.executable == process.executable)
        {
            return Ok(());
        }
        if self.process_fact_count >= MAX_PROCESS_FACTS {
            return Err("process-history-bound".into());
        }
        // exec preserves the PID. Keep each observed transition, including the
        // compiler behind a wrapper, without copying identical poll records.
        self.process_fact_count = self.process_fact_count.checked_add(1).ok_or("process-history-bound")?;
        history.push(process);
        assert!(self.process_fact_count <= MAX_PROCESS_FACTS);
        Ok(())
    }

    fn check_disk(&mut self) -> Result<()> {
        if self.disk_check.elapsed() >= Duration::from_millis(DISK_CHECK_MS) {
            tree_bytes(self.scratch, &mut 0, &mut 0)?;
            self.disk_check = Instant::now();
        }
        Ok(())
    }

    fn run(
        &mut self,
        owned: &mut Owned,
        files: &OutputFiles,
        observer: &mut dyn FnMut(ProcessEvent),
        trace: &mut exec::Trace,
    ) -> Result<Option<ExitStatus>> {
        let pid = i32::try_from(owned.0.id()).map_err(err)?;
        assert!(pid > 0);
        for _ in 0..MAX_MONITOR_POLLS {
            if let Err(error) = trace.drain(&mut |process| self.observe_exec(process, pid)) {
                self.failure = Some(error);
                return Ok(None);
            }
            if let Err(error) = self.observe(pid) {
                self.failure = Some(error);
                return Ok(None);
            }
            if self.deadline_expired() {
                self.failure = Some("deadline".into());
                return Ok(None);
            }
            if files.exceeded()? {
                self.failure = Some("stream-bound".into());
                return Ok(None);
            }
            if let Err(error) = self.check_disk() {
                self.failure = Some(error);
                return Ok(None);
            }
            if let Some(status) = trace.status() {
                emit_exit(observer, status, false);
                assert!(self.failure.is_none());
                return Ok(Some(status));
            }
            thread::sleep(Duration::from_millis(POLL_MS));
        }
        // The fixed pass budget cannot precede TOTAL_MS: every continuing pass
        // slept POLL_MS. Retain the same timeout outcome at budget exhaustion.
        self.failure = Some("deadline".into());
        Ok(None)
    }
}

fn inject_compiler(
    injection: &Injection,
    observed: &[ProcessFact],
    pid: i32,
    origin_ns: u64,
) -> Result<Option<Trigger>> {
    assert!(pid > 0);
    assert!(observed.len() <= PROCESS_MAX);
    for process in observed {
        if process.executable.as_ref() != Some(&injection.rustc) {
            continue;
        }
        let environment =
            read_process_bytes(&PathBuf::from(format!("/proc/{}/environ", process.pid)))?.unwrap_or_default();
        let expected = format!("CARGO_MANIFEST_DIR={}", injection.manifest.display());
        if !environment.split(|byte| *byte == 0).any(|value| value == expected.as_bytes()) {
            continue;
        }
        // Signal only a still-running exact executable/package compiler, never a timer proxy.
        let signal = if injection.fault == Fault::Crash {
            libc::SIGKILL
        } else {
            libc::SIGTERM
        };
        let is_delivered = if injection.fault == Fault::Crash {
            i32::try_from(process.pid).is_ok_and(|pid| unsafe { libc::kill(pid, signal) } == 0)
        } else {
            signal_group(GroupSignal { pid, signal }).is_ok()
        };
        return Ok(Some(Trigger {
            pid: process.pid,
            executable: injection.rustc.clone(),
            command: process.command.clone(),
            observed_ns: clock_offset(origin_ns),
            signal,
            delivered: is_delivered,
        }));
    }
    Ok(None)
}

fn emit_exit(observer: &mut dyn FnMut(ProcessEvent), status: ExitStatus, timeout: bool) {
    observer(ProcessEvent::Exited {
        code: status.code(),
        signal: status.signal(),
        timeout,
    });
}

fn terminate_group(pid: i32) -> Result<()> {
    assert!(pid > 0);
    const { assert!(TERMINATE_GRACE_MS < TEARDOWN_MS) };
    signal_group(GroupSignal {
        pid,
        signal: libc::SIGTERM,
    })?;
    let grace = Instant::now();
    for _ in 0..=TERMINATE_GRACE_MS / POLL_MS {
        if grace.elapsed() >= Duration::from_millis(TERMINATE_GRACE_MS) {
            break;
        }
        let is_settled = match group(pid) {
            Ok(members) => members.is_empty(),
            // A group too large to observe gets no further grace before SIGKILL.
            Err(_) => true,
        };
        if is_settled {
            break;
        }
        thread::sleep(Duration::from_millis(POLL_MS));
    }
    signal_group(GroupSignal {
        pid,
        signal: libc::SIGKILL,
    })
}

fn reap_group(pid: i32) -> Result<bool> {
    assert!(pid > 0);
    let teardown_start = Instant::now();
    let mut reaped_count = 0;
    for _ in 0..MAX_TEARDOWN_POLLS {
        let mut status = 0;
        // Every owned member is traced, so traced threads need __WALL to be reaped.
        let reaped = unsafe { libc::waitpid(-pid, &mut status, libc::WNOHANG | libc::__WALL) };
        // A group still too large to observe is not empty yet; the deadline decides.
        if group(pid).is_ok_and(|members| members.is_empty()) {
            return Ok(true);
        }
        if teardown_start.elapsed() >= Duration::from_millis(TEARDOWN_MS) {
            return Ok(false);
        }
        if reaped <= 0 {
            thread::sleep(Duration::from_millis(POLL_MS));
        } else {
            reaped_count += 1;
            if reaped_count > PROCESS_MAX {
                return Err("process-count-bound".into());
            }
        }
        assert!(reaped_count <= PROCESS_MAX);
    }
    Ok(false)
}

struct Completion {
    status: ExitStatus,
    elapsed_ns: u64,
    last_reap_ns: u64,
    teardown_complete: bool,
}

fn finish_child(
    owned: &mut Owned,
    monitor: &mut Monitor<'_>,
    status: Option<ExitStatus>,
    observer: &mut dyn FnMut(ProcessEvent),
    trace: &mut exec::Trace,
) -> Result<Completion> {
    let elapsed_ns = clock_offset(monitor.origin_ns);
    let pid = i32::try_from(owned.0.id()).map_err(err)?;
    assert!(pid > 0);
    terminate_group(pid)?;
    let teardown_start = Instant::now();
    for _ in 0..MAX_TEARDOWN_POLLS {
        let drained = trace.drain(&mut |process| monitor.record_process(process));
        if let Err(error) = drained {
            // Teardown still completes; the first observation failure stays the reported one.
            monitor.failure.get_or_insert(error);
        }
        // Untracked traced threads keep the leader unreapable until reaped here.
        trace.reap_killed_group();
        if trace.is_empty() || teardown_start.elapsed() >= Duration::from_millis(TEARDOWN_MS) {
            break;
        }
        thread::sleep(Duration::from_millis(POLL_MS));
    }
    let final_status = trace.status().ok_or("exec-observer-root-not-reaped")?;
    assert!(!(final_status.code().is_some() && final_status.signal().is_some()));
    if status.is_none() {
        emit_exit(observer, final_status, monitor.failure.as_deref() == Some("deadline"));
    }
    let is_teardown_complete = trace.is_empty() && reap_group(pid)?;
    let last_reap_ns = clock_offset(monitor.origin_ns);
    if is_teardown_complete {
        observer(ProcessEvent::Reaped);
    }
    Ok(Completion {
        status: status.unwrap_or(final_status),
        elapsed_ns,
        last_reap_ns,
        teardown_complete: is_teardown_complete,
    })
}

fn capture(
    description: CommandDescription,
    files: OutputFiles,
    mut monitor: Monitor<'_>,
    completion: Completion,
) -> Result<Capture> {
    let stdout = files.prefix(&files.stdout)?;
    let stderr = files.prefix(&files.stderr)?;
    assert!(u64::try_from(stdout.len()).map_err(err)? <= files.bound_bytes);
    assert!(u64::try_from(stderr.len()).map_err(err)? <= files.bound_bytes);
    if files.exceeded()? {
        monitor.failure = Some("stream-bound".into());
    }
    Ok(Capture {
        argv: description.argv,
        environment: description.environment,
        cwd: description.cwd,
        stdout,
        stderr,
        code: completion.status.code(),
        signal: completion.status.signal(),
        monotonic_origin_ns: monitor.origin_ns,
        elapsed_ns: completion.elapsed_ns,
        last_reap_ns: completion.last_reap_ns,
        complete: monitor.failure.is_none(),
        teardown_complete: completion.teardown_complete,
        failure: monitor.failure,
        trigger: monitor.trigger,
        processes: monitor.processes.into_values().flatten().collect(),
        resources: monitor.sampler.finish(completion.last_reap_ns),
    })
}

pub fn supervise_observed(
    mut command: Command,
    options: Supervision<'_>,
    observer: &mut dyn FnMut(ProcessEvent),
) -> Result<Capture> {
    let Supervision {
        root,
        scratch,
        timeout_ms,
        total_start,
        injection,
        stream_bound,
    } = options;
    let description = CommandDescription::read(&command)?;
    let files = OutputFiles::attach(&mut command, root, stream_bound)?;
    configure_child(&mut command);
    // A successful exec stop is independent of polling frequency and compiler duration.
    unsafe {
        command.pre_exec(exec::prepare);
    }
    let origin_ns = crate::waits::monotonic_ns().map_err(err)?;
    let started = Instant::now();
    let mut owned = Owned(command.spawn().map_err(err)?);
    assert!(owned.0.id() > 0);
    assert!(i32::try_from(owned.0.id()).is_ok());
    observer(ProcessEvent::Spawned(owned.0.id()));
    let mut monitor = Monitor {
        scratch,
        timeout_ms,
        total_start,
        injection,
        started,
        disk_check: Instant::now(),
        origin_ns,
        trigger: None,
        failure: None,
        processes: BTreeMap::new(),
        process_fact_count: 0,
        sampler: crate::waits::ProcessTreeSampler::new(owned.0.id()),
    };
    let mut trace = exec::Trace::new(i32::try_from(owned.0.id()).map_err(err)?);
    let status = monitor.run(&mut owned, &files, observer, &mut trace)?;
    let completion = finish_child(&mut owned, &mut monitor, status, observer, &mut trace)?;
    capture(description, files, monitor, completion)
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Instant;

    use super::CHILD_TASKS_MAX;
    use super::ProcessTasks;
    use super::admit_process_tasks;

    const THREAD_HELPER: &str = "MANTLE_OBSERVED_PROCESS_TEST_THREADS";

    /// Re-executed by the teardown regression below; inert in an ordinary test run.
    #[test]
    fn thread_spawning_helper() {
        let Some(count) = std::env::var(THREAD_HELPER).ok().and_then(|value| value.parse::<u32>().ok()) else {
            return;
        };
        let threads: Vec<_> = (0..count)
            .map(|_| std::thread::spawn(|| std::thread::sleep(std::time::Duration::from_secs(60))))
            .collect();
        std::thread::sleep(std::time::Duration::from_secs(60));
        drop(threads);
    }

    #[test]
    fn traced_threads_beyond_the_task_bound_are_reaped_without_blocking() {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "observed_process::tests::thread_spawning_helper",
                "--test-threads=1",
            ])
            .env(THREAD_HELPER, CHILD_TASKS_MAX.saturating_mul(2).to_string())
            .current_dir(root.path());
        let started = Instant::now();
        let capture = super::supervise(command, super::Supervision {
            root: root.path(),
            scratch: root.path(),
            timeout_ms: 20_000,
            total_start: Instant::now(),
            injection: None,
            stream_bound: super::STREAM_BYTES,
        })
        .unwrap();
        assert!(!capture.complete, "exceeding the task bound is not complete observation");
        assert!(capture.failure.is_some());
        assert!(capture.teardown_complete, "every traced thread and process was reaped");
        assert!(started.elapsed() < std::time::Duration::from_secs(60), "teardown is bounded");
    }
    #[test]
    fn short_lived_descendant_execs_survive_until_independently_observed() {
        let root = tempfile::tempdir().unwrap();
        // Resolve the declared executable from PATH: hosts need not provide /bin/true.
        let path = std::env::var_os("PATH").unwrap();
        let executable = std::env::split_paths(&path)
            .map(|directory| directory.join("true"))
            .find(|candidate| candidate.is_file())
            .expect("test executable `true` is on PATH");
        let expected = std::fs::canonicalize(&executable).unwrap();
        let mut command = Command::new("sh");
        command
            .args(["-c", "\"$1\" & \"$1\" & wait; exit 7", "fixture"])
            .arg(&executable)
            .current_dir(root.path());
        let capture = super::supervise(command, super::Supervision {
            root: root.path(),
            scratch: root.path(),
            timeout_ms: super::PROBE_MS,
            total_start: Instant::now(),
            injection: None,
            stream_bound: super::STREAM_BYTES,
        })
        .unwrap();
        assert_eq!(capture.code, Some(7));
        assert_eq!(capture.signal, None);
        assert!(capture.complete, "{:?}", capture.failure);
        assert!(capture.teardown_complete);
        let children: std::collections::BTreeSet<_> = capture
            .processes
            .iter()
            .filter(|process| process.executable.as_ref() == Some(&expected))
            .map(|process| process.pid)
            .collect();
        assert_eq!(children.len(), 2, "both actual short-lived execs need independent process facts");
    }

    #[test]
    fn observed_signal_exit_is_not_converted_to_an_exit_code() {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "kill -TERM $$"]).current_dir(root.path());
        let capture = super::supervise(command, super::Supervision {
            root: root.path(),
            scratch: root.path(),
            timeout_ms: super::PROBE_MS,
            total_start: Instant::now(),
            injection: None,
            stream_bound: super::STREAM_BYTES,
        })
        .unwrap();
        assert_eq!(capture.code, None);
        assert_eq!(capture.signal, Some(libc::SIGTERM));
        assert!(capture.complete, "{:?}", capture.failure);
        assert!(capture.teardown_complete);
    }

    #[test]
    fn a_stopped_child_remains_deadline_bounded_and_is_reaped() {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "kill -STOP $$"]).current_dir(root.path());
        let capture = super::supervise(command, super::Supervision {
            root: root.path(),
            scratch: root.path(),
            timeout_ms: 500,
            total_start: Instant::now(),
            injection: None,
            stream_bound: super::STREAM_BYTES,
        })
        .unwrap();
        assert_eq!(capture.failure.as_deref(), Some("deadline"));
        assert_eq!(capture.code, None);
        assert_eq!(capture.signal, Some(libc::SIGKILL));
        assert!(!capture.complete);
        assert!(capture.teardown_complete);
    }

    #[test]
    fn owned_task_bound_counts_all_group_members_without_uid_interference() {
        let first = admit_process_tasks(ProcessTasks {
            total: 0,
            observed: CHILD_TASKS_MAX - 1,
        })
        .unwrap();
        assert_eq!(
            admit_process_tasks(ProcessTasks {
                total: first,
                observed: 1
            })
            .unwrap(),
            CHILD_TASKS_MAX
        );
        assert!(
            admit_process_tasks(ProcessTasks {
                total: first,
                observed: 2
            })
            .is_err()
        );
        assert!(admit_process_tasks(ProcessTasks { total: 0, observed: 0 }).is_err());
        assert!(
            admit_process_tasks(ProcessTasks {
                total: first,
                observed: u32::MAX
            })
            .is_err()
        );
    }
}
