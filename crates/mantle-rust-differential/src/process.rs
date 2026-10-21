//! Linux-only bounded private filesystem and child process mechanics.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
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
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use mantle_rust_differential_core::Digest;
use serde::Deserialize;
use serde::Serialize;

use crate::Result;
use crate::err;
use crate::fixtures::Snapshot;

pub const STREAM_BYTES: u64 = 8 * 1024 * 1024;
pub const SOURCE_BYTES: u64 = 2 * 1024 * 1024;
pub const SCRATCH_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const FILES_MAX: u32 = 200_000;
pub const PROCESS_MAX: usize = 64;
pub const BUILD_MS: u64 = 120_000;
pub const PROGRAM_MS: u64 = 5_000;
pub const PROBE_MS: u64 = 10_000;
pub const TOTAL_MS: u64 = 7_200_000;
pub const TEARDOWN_MS: u64 = 5_000;
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
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
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
    fs::DirBuilder::new().mode(0o700).create(path).map_err(err)?;
    let metadata = fs::symlink_metadata(path).map_err(err)?;
    if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o777 != 0o700 {
        return Err("private-root-owner-mode".into());
    }
    Ok(())
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(err)?;
    file.write_all(bytes).map_err(err)?;
    file.sync_all().map_err(err)
}
pub fn read_bounded(path: &Path, bound: u64) -> Result<Vec<u8>> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(err)?;
    let metadata = file.metadata().map_err(err)?;
    if !metadata.is_file() || metadata.len() > bound {
        return Err(format!("file-bound:{}", path.display()));
    }
    let mut bytes = Vec::new();
    file.take(bound + 1).read_to_end(&mut bytes).map_err(err)?;
    if bytes.len() as u64 > bound {
        return Err("file-grew-beyond-bound".into());
    }
    Ok(bytes)
}
fn relative(path: &str) -> Result<&Path> {
    let path = Path::new(path);
    if path.as_os_str().is_empty() || path.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("unsafe-snapshot-member".into());
    }
    Ok(path)
}
pub fn materialize(root: &Path, snapshot: &Snapshot) -> Result<()> {
    private_dir(root)?;
    let mut total = 0u64;
    for (name, bytes) in snapshot {
        total = total.checked_add(bytes.len() as u64).ok_or("source-size-overflow")?;
        if total > SOURCE_BYTES || snapshot.len() > 128 {
            return Err("source-bound".into());
        }
        let dest = root.join(relative(name)?);
        let mut current = root.to_path_buf();
        for component in relative(name)?.parent().ok_or("member-parent")?.components() {
            current.push(component);
            if !current.exists() {
                private_dir(&current)?;
            } else if !fs::symlink_metadata(&current).map_err(err)?.is_dir() {
                return Err("snapshot-parent-type".into());
            }
        }
        write_new(&dest, bytes)?;
    }
    verify(root, snapshot)
}
fn inventory_inner(root: &Path, prefix: &Path, files: &mut BTreeMap<String, Vec<u8>>, total: &mut u64) -> Result<()> {
    for entry in fs::read_dir(root.join(prefix)).map_err(err)? {
        let entry = entry.map_err(err)?;
        let p = prefix.join(entry.file_name());
        let m = entry.metadata().map_err(err)?;
        if entry.file_type().map_err(err)?.is_symlink() {
            return Err("snapshot-symlink".into());
        }
        if m.is_dir() {
            inventory_inner(root, &p, files, total)?;
        } else if m.is_file() {
            *total = total.checked_add(m.len()).ok_or("source-overflow")?;
            if *total > SOURCE_BYTES || files.len() >= 128 {
                return Err("source-inventory-bound".into());
            }
            files.insert(p.to_str().ok_or("nonutf8-member")?.into(), read_bounded(&root.join(p), SOURCE_BYTES)?);
        } else {
            return Err("snapshot-special-file".into());
        }
    }
    Ok(())
}
pub fn snapshot(root: &Path) -> Result<Snapshot> {
    let mut s = Snapshot::new();
    inventory_inner(root, Path::new(""), &mut s, &mut 0)?;
    Ok(s)
}
pub fn verify(root: &Path, expected: &Snapshot) -> Result<()> {
    if snapshot(root)? != *expected {
        return Err("snapshot-postimage-mismatch".into());
    }
    Ok(())
}
pub fn apply(root: &Path, before: &Snapshot, step: &crate::fixtures::Step) -> Result<()> {
    verify(root, before)?;
    for edit in &step.edits {
        if edit.matches != 1 || before.get(&edit.path) != edit.before.as_ref() {
            return Err("edit-preimage-or-count".into());
        }
        let path = root.join(relative(&edit.path)?);
        if edit.before.is_some() {
            if read_bounded(&path, SOURCE_BYTES)? != *edit.before.as_ref().ok_or("edit-before")? {
                return Err("edit-missed".into());
            }
            fs::remove_file(&path).map_err(err)?;
        }
        if let Some(bytes) = &edit.after {
            let parent = path.parent().ok_or("edit-parent")?;
            if !parent.exists() {
                private_dir(parent)?;
            }
            write_new(&path, bytes)?;
        }
    }
    verify(root, &step.snapshot)
}
pub fn tree_bytes(root: &Path, files: &mut u32, total: &mut u64) -> Result<()> {
    for e in fs::read_dir(root).map_err(err)? {
        let e = e.map_err(err)?;
        *files = files.checked_add(1).ok_or("file-count-overflow")?;
        if *files > FILES_MAX {
            return Err("scratch-file-bound".into());
        }
        let t = e.file_type().map_err(err)?;
        if t.is_dir() {
            tree_bytes(&e.path(), files, total)?;
        } else {
            *total = total.checked_add(e.metadata().map_err(err)?.len()).ok_or("scratch-overflow")?;
        }
        if *total > SCRATCH_BYTES {
            return Err("scratch-byte-bound".into());
        }
    }
    Ok(())
}
/// Hash complete declared closures, with bounded traversal and explicit symlink bytes.
pub fn hash_closure(root: &Path) -> Result<Digest> {
    fn visit(path: &Path, logical: &Path, h: &mut blake3::Hasher, n: &mut u32, bytes: &mut u64) -> Result<()> {
        *n += 1;
        if *n > 200_000 {
            return Err("tool-closure-file-bound".into());
        }
        let m = fs::symlink_metadata(path).map_err(err)?;
        h.update(logical.as_os_str().as_encoded_bytes());
        h.update(&[0]);
        if m.file_type().is_symlink() {
            h.update(b"link");
            h.update(fs::read_link(path).map_err(err)?.as_os_str().as_encoded_bytes());
            return Ok(());
        }
        if m.is_dir() {
            let mut members = fs::read_dir(path)
                .map_err(err)?
                .map(|e| e.map(|e| e.file_name()))
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(err)?;
            members.sort();
            for name in members {
                visit(&path.join(&name), &logical.join(&name), h, n, bytes)?;
            }
        } else if m.is_file() {
            *bytes = bytes.checked_add(m.len()).ok_or("closure-size")?;
            if *bytes > 16 * 1024 * 1024 * 1024 {
                return Err("tool-closure-byte-bound".into());
            }
            let mut file = File::open(path).map_err(err)?;
            let mut buffer = [0u8; 65536];
            loop {
                let size = file.read(&mut buffer).map_err(err)?;
                if size == 0 {
                    break;
                }
                h.update(&buffer[..size]);
            }
        } else {
            return Err("tool-closure-special-file".into());
        }
        Ok(())
    }
    let mut hasher = blake3::Hasher::new();
    visit(root, Path::new(""), &mut hasher, &mut 0, &mut 0)?;
    Digest::parse(hasher.finalize().to_hex().to_string()).map_err(err)
}
fn group(pid: i32) -> Result<Vec<ProcessFact>> {
    let mut found = vec![];
    for e in fs::read_dir("/proc").map_err(err)? {
        let e = e.map_err(err)?;
        let Ok(id) = e.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Ok(stat) = fs::read_to_string(e.path().join("stat")) else {
            continue;
        };
        let Some((_, tail)) = stat.rsplit_once(") ") else {
            continue;
        };
        if tail.split_whitespace().nth(2).and_then(|n| n.parse::<i32>().ok()) != Some(pid) {
            continue;
        }
        if found.len() >= PROCESS_MAX {
            return Err("process-count-bound".into());
        }
        let command = fs::read(e.path().join("cmdline")).unwrap_or_default();
        found.push(ProcessFact {
            pid: id,
            command: Digest::from_slice(&command),
            executable: fs::read_link(e.path().join("exe")).ok(),
        });
    }
    Ok(found)
}
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGKILL);
        }
        let _ = self.0.wait();
    }
}
fn signal_group(pid: i32, signal: i32) -> Result<()> {
    if unsafe { libc::kill(-pid, signal) } != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
        return Err("group-signal".into());
    }
    Ok(())
}
pub fn supervise(
    mut command: Command,
    root: &Path,
    scratch: &Path,
    timeout_ms: u64,
    total_start: Instant,
    injection: Option<Injection>,
    stream_bound: u64,
) -> Result<Capture> {
    let cwd = command.get_current_dir().ok_or("command-cwd")?.to_path_buf();
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let environment = command
        .get_envs()
        .filter_map(|(k, v)| v.map(|v| (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned())))
        .collect();
    let stdout_path = root.join("stdout.raw");
    let stderr_path = root.join("stderr.raw");
    let open = |path: &Path| {
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(err)
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(open(&stdout_path)?))
        .stderr(Stdio::from(open(&stderr_path)?));
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            for (kind, value) in [
                (libc::RLIMIT_AS, 8 * 1024 * 1024 * 1024u64),
                (libc::RLIMIT_CPU, 120),
                (libc::RLIMIT_FSIZE, 128 * 1024 * 1024),
                (libc::RLIMIT_NOFILE, 256),
                (libc::RLIMIT_NPROC, 128),
                (libc::RLIMIT_CORE, 0),
            ] {
                let limit = libc::rlimit {
                    rlim_cur: value,
                    rlim_max: value,
                };
                if libc::setrlimit(kind, &limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    let monotonic_origin_ns = crate::waits::monotonic_ns().map_err(err)?;
    let started = Instant::now();
    let mut owned = Owned(command.spawn().map_err(err)?);
    let pid = owned.0.id() as i32;
    let mut sampler = crate::waits::ProcessTreeSampler::new(owned.0.id());
    let mut trigger = None;
    let mut failure = None;
    let mut processes = BTreeMap::new();
    let mut disk_check = Instant::now();
    let status = loop {
        let observed = match group(pid) {
            Ok(p) => p,
            Err(e) => {
                failure = Some(e);
                break None;
            }
        };
        sampler.sample(clock_offset(monotonic_origin_ns));
        for p in &observed {
            processes.entry(p.pid).or_insert_with(|| p.clone());
        }
        if let Some(inject) = &injection
            && trigger.is_none()
        {
            for p in &observed {
                if p.executable.as_ref() != Some(&inject.rustc) {
                    continue;
                }
                let env = fs::read(format!("/proc/{}/environ", p.pid)).unwrap_or_default();
                let expected = format!("CARGO_MANIFEST_DIR={}", inject.manifest.display());
                if !env.split(|b| *b == 0).any(|v| v == expected.as_bytes()) {
                    continue;
                }
                // Signal only a still-running exact executable/package compiler, never a timer proxy.
                let signal = if inject.fault == Fault::Crash {
                    libc::SIGKILL
                } else {
                    libc::SIGTERM
                };
                let delivered = if inject.fault == Fault::Crash {
                    (unsafe { libc::kill(p.pid as i32, signal) }) == 0
                } else {
                    signal_group(pid, signal).is_ok()
                };
                trigger = Some(Trigger {
                    pid: p.pid,
                    executable: inject.rustc.clone(),
                    command: p.command.clone(),
                    observed_ns: clock_offset(monotonic_origin_ns),
                    signal,
                    delivered,
                });
                break;
            }
        }
        if started.elapsed() >= Duration::from_millis(timeout_ms)
            || total_start.elapsed() >= Duration::from_millis(TOTAL_MS)
        {
            failure = Some("deadline".into());
            break None;
        }
        if fs::metadata(&stdout_path).map_err(err)?.len() > stream_bound
            || fs::metadata(&stderr_path).map_err(err)?.len() > stream_bound
        {
            failure = Some("stream-bound".into());
            break None;
        }
        if disk_check.elapsed() >= Duration::from_millis(100) {
            if let Err(e) = tree_bytes(scratch, &mut 0, &mut 0) {
                failure = Some(e);
                break None;
            }
            disk_check = Instant::now();
        }
        if let Some(status) = owned.0.try_wait().map_err(err)? {
            break Some(status);
        }
        thread::sleep(Duration::from_millis(2));
    };
    let elapsed_ns = clock_offset(monotonic_origin_ns);
    signal_group(pid, libc::SIGTERM)?;
    let grace = Instant::now();
    while grace.elapsed() < Duration::from_millis(100) && !group(pid)?.is_empty() {
        thread::sleep(Duration::from_millis(2));
    }
    signal_group(pid, libc::SIGKILL)?;
    let final_status = owned.0.wait().map_err(err)?;
    let teardown_start = Instant::now();
    let mut teardown_complete = false;
    loop {
        let mut status = 0;
        let reaped = unsafe { libc::waitpid(-pid, &mut status, libc::WNOHANG) };
        if group(pid)?.is_empty() {
            teardown_complete = true;
            break;
        }
        if teardown_start.elapsed() >= Duration::from_millis(TEARDOWN_MS) {
            break;
        }
        if reaped <= 0 {
            thread::sleep(Duration::from_millis(2));
        }
    }
    let last_reap_ns = clock_offset(monotonic_origin_ns);
    let mut complete = failure.is_none();
    let prefix = |p: &Path| -> Result<Vec<u8>> {
        let mut b = vec![];
        File::open(p).map_err(err)?.take(stream_bound).read_to_end(&mut b).map_err(err)?;
        Ok(b)
    };
    let stdout = prefix(&stdout_path)?;
    let stderr = prefix(&stderr_path)?;
    if fs::metadata(&stdout_path).map_err(err)?.len() > stream_bound
        || fs::metadata(&stderr_path).map_err(err)?.len() > stream_bound
    {
        complete = false;
        failure = Some("stream-bound".into());
    }
    let status = status.unwrap_or(final_status);
    Ok(Capture {
        argv,
        environment,
        cwd,
        stdout,
        stderr,
        code: status.code(),
        signal: status.signal(),
        monotonic_origin_ns,
        elapsed_ns,
        last_reap_ns,
        complete,
        teardown_complete,
        failure,
        trigger,
        processes: processes.into_values().collect(),
        resources: sampler.finish(last_reap_ns),
    })
}
/// Remove only an already-finalized private reference tree, never a retained campaign.
pub fn remove_owned_tree(root: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(root).map_err(err)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.uid() != unsafe { libc::geteuid() } {
        return Err("cleanup-root-ownership".into());
    }
    fn remove(path: &Path, count: &mut u32) -> Result<()> {
        for entry in fs::read_dir(path).map_err(err)? {
            let entry = entry.map_err(err)?;
            *count = count.checked_add(1).ok_or("cleanup-overflow")?;
            if *count > FILES_MAX {
                return Err("cleanup-file-bound".into());
            }
            if entry.file_type().map_err(err)?.is_dir() {
                remove(&entry.path(), count)?;
                fs::remove_dir(entry.path()).map_err(err)?;
            } else {
                fs::remove_file(entry.path()).map_err(err)?;
            }
        }
        Ok(())
    }
    remove(root, &mut 0)?;
    fs::remove_dir(root).map_err(err)
}
