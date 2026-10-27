//! Exact-child worker measurements. No process-wide child counters or exit-code
//! heuristics participate in accounting or OOM authority.
use std::io::Read;
use std::io::{self};
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use super::RemoteProductionAttemptBinding;

pub const MAX_USAGE_REPORT_BYTES: usize = 4096;
pub const MAX_USAGE_STDOUT_BYTES: usize = 67_108_864;
pub const MAX_USAGE_STDERR_BYTES: usize = 65_536;
const MAX_USAGE_WALL_SECS: u64 = 86_400;
const MAX_USAGE_OUTPUT_BYTES: u64 = 1 << 50;
const USAGE_DOMAIN: &str = "mantle-exact-child-usage-v1";

/// Positive OOM evidence requires a fresh dedicated delegated kernel cgroup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteUsageOomEvidence {
    Unavailable,
    DedicatedCgroupNoOom,
    DedicatedCgroupOom { oom_kill_delta: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteUsageCompleteness {
    ProcessOnly,
    DedicatedCgroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteScratchCoverage {
    Unavailable,
    Sampled,
    TraversalIncomplete,
}

/// Allocated file blocks in the executor's private temporary workspace only;
/// store mounts and symlinks are excluded. The maximum of bounded snapshots is
/// a sampled lower bound, never an exact high-water mark or history authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteScratchObservation {
    pub sampled_peak_bytes: Option<u64>,
    pub end_snapshot_bytes: Option<u64>,
    pub samples: u32,
    pub coverage: RemoteScratchCoverage,
}

impl Default for RemoteScratchObservation {
    fn default() -> Self {
        Self {
            sampled_peak_bytes: None,
            end_snapshot_bytes: None,
            samples: 0,
            coverage: RemoteScratchCoverage::Unavailable,
        }
    }
}

pub fn failure_result_digest(binding: &RemoteProductionAttemptBinding, request_id: &str) -> String {
    let mut hasher = blake3::Hasher::new_derive_key("mantle-worker-failed-result-v1");
    for part in [binding.job_id.as_str(), binding.attempt_id.as_str(), request_id] {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.update(&binding.fence_generation.get().to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsageFacts {
    schema: String,
    binding: RemoteProductionAttemptBinding,
    request_id: String,
    result_digest_blake3: String,
    child_pid: u32,
    cpu_time_millis: u64,
    wall_time_millis: u64,
    peak_rss_bytes: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    exit_code: Option<i32>,
    term_signal: Option<i32>,
    oom: RemoteUsageOomEvidence,
    completeness: RemoteUsageCompleteness,
    cgroup_peak_memory_bytes: Option<u64>,
    scratch: RemoteScratchObservation,
}

/// Serializable, untrusted until `verify` checks both the signature and the
/// current attempt/request/result. Deserialization never produces authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteSignedUsageReport {
    facts: UsageFacts,
    signature: String,
}

/// Host-created authenticated observation. No constructor or Deserialize.
/// ProcessOnly is partial: CPU/RSS describe the exact wait4 child. Dedicated
/// cgroup CPU covers its fully terminated subtree; memory.peak is a tree peak,
/// not an integral. Output means observed stdout + stderr bytes. Neither scope
/// makes sampled scratch an exact peak or supplies queue/I/O measurements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAuthenticatedUsage {
    facts: UsageFacts,
}

impl RemoteAuthenticatedUsage {
    pub fn binding(&self) -> &RemoteProductionAttemptBinding {
        &self.facts.binding
    }
    pub fn request_id(&self) -> &str {
        &self.facts.request_id
    }
    pub fn result_digest_blake3(&self) -> &str {
        &self.facts.result_digest_blake3
    }
    pub fn cpu_time_millis(&self) -> u64 {
        self.facts.cpu_time_millis
    }
    pub fn wall_time_millis(&self) -> u64 {
        self.facts.wall_time_millis
    }
    pub fn peak_rss_bytes(&self) -> u64 {
        self.facts.peak_rss_bytes
    }
    pub fn output_bytes(&self) -> u64 {
        self.facts.stdout_bytes + self.facts.stderr_bytes
    }
    pub fn stdout_bytes(&self) -> u64 {
        self.facts.stdout_bytes
    }
    pub fn stderr_bytes(&self) -> u64 {
        self.facts.stderr_bytes
    }
    pub fn oom_evidence(&self) -> RemoteUsageOomEvidence {
        self.facts.oom
    }
    pub fn measurement_completeness(&self) -> RemoteUsageCompleteness {
        self.facts.completeness
    }
    pub fn cgroup_peak_memory_bytes(&self) -> Option<u64> {
        self.facts.cgroup_peak_memory_bytes
    }
    pub fn scratch_observation(&self) -> &RemoteScratchObservation {
        &self.facts.scratch
    }
    pub fn event_digest_blake3(&self) -> String {
        // A fixed struct of bounded scalar fields has infallible JSON encoding.
        let bytes = serde_json::to_vec(&self.facts).expect("usage scalar serialization");
        blake3::hash(&bytes).to_hex().to_string()
    }
    pub fn validate_current(
        &self,
        binding: &RemoteProductionAttemptBinding,
        request_id: &str,
        result_digest_blake3: &str,
    ) -> Result<(), String> {
        validate_facts(&self.facts)?;
        if &self.facts.binding != binding
            || self.request_id() != request_id
            || self.result_digest_blake3() != result_digest_blake3
        {
            return Err("remote-usage-current-attempt-mismatch".into());
        }
        Ok(())
    }
    pub fn sign(&self, keypair: &crunch_build::KeyPair) -> Result<RemoteSignedUsageReport, String> {
        validate_facts(&self.facts)?;
        let fingerprint = fingerprint(&self.facts)?;
        let signature = keypair.signing_key.sign(fingerprint.as_bytes()).to_string();
        let report = RemoteSignedUsageReport {
            facts: self.facts.clone(),
            signature,
        };
        if serde_json::to_vec(&report).map_err(|_| "remote-usage-encode")?.len() > MAX_USAGE_REPORT_BYTES {
            return Err("remote-usage-report-too-large".into());
        }
        Ok(report)
    }
}

impl RemoteSignedUsageReport {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_USAGE_REPORT_BYTES {
            return Err("remote-usage-report-too-large".into());
        }
        serde_json::from_slice(bytes).map_err(|_| "remote-usage-report-invalid".into())
    }
    pub fn verify(
        &self,
        binding: &RemoteProductionAttemptBinding,
        request_id: &str,
        result_digest_blake3: &str,
        key: &nix_compat::narinfo::VerifyingKey,
    ) -> Result<RemoteAuthenticatedUsage, String> {
        validate_facts(&self.facts)?;
        if self.signature.len() > 512 {
            return Err("remote-usage-signature-invalid".into());
        }
        let signature =
            nix_compat::narinfo::SignatureRef::parse(&self.signature).map_err(|_| "remote-usage-signature-invalid")?;
        if !key.verify(&fingerprint(&self.facts)?, &signature) {
            return Err("remote-usage-signature-invalid".into());
        }
        let usage = RemoteAuthenticatedUsage {
            facts: self.facts.clone(),
        };
        usage.validate_current(binding, request_id, result_digest_blake3)?;
        Ok(usage)
    }
}

fn fingerprint(facts: &UsageFacts) -> Result<String, String> {
    let bytes = serde_json::to_vec(facts).map_err(|_| "remote-usage-encode")?;
    Ok(format!("{USAGE_DOMAIN}:{}", blake3::hash(&bytes).to_hex()))
}

fn validate_identity(binding: &RemoteProductionAttemptBinding, request_id: &str) -> Result<(), String> {
    use crunch_remote_core::attempt::RemoteAttemptId;
    use crunch_remote_core::attempt::RemoteJobId;
    RemoteJobId::new(binding.job_id.as_str()).map_err(|_| "remote-usage-binding-invalid")?;
    RemoteAttemptId::new(binding.attempt_id.as_str()).map_err(|_| "remote-usage-binding-invalid")?;
    if binding.fence_generation.get() == 0
        || request_id.is_empty()
        || request_id.len() > 128
        || !request_id.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
    {
        return Err("remote-usage-binding-invalid".into());
    }
    Ok(())
}

fn validate_facts(f: &UsageFacts) -> Result<(), String> {
    validate_identity(&f.binding, &f.request_id)?;
    if f.schema != USAGE_DOMAIN
        || !super::is_blake3_hex_digest(&f.result_digest_blake3)
        || f.child_pid == 0
        || f.wall_time_millis > MAX_USAGE_WALL_SECS * 1000 + 1000
        || f.peak_rss_bytes > (1 << 60)
        || f.stdout_bytes.checked_add(f.stderr_bytes).is_none_or(|n| n > MAX_USAGE_OUTPUT_BYTES)
        || matches!(f.oom, RemoteUsageOomEvidence::DedicatedCgroupOom { oom_kill_delta: 0 })
        || (f.completeness == RemoteUsageCompleteness::ProcessOnly
            && (f.oom != RemoteUsageOomEvidence::Unavailable || f.cgroup_peak_memory_bytes.is_some()))
        || f.scratch.samples > 1_000_000
        || (f.completeness == RemoteUsageCompleteness::DedicatedCgroup && f.oom == RemoteUsageOomEvidence::Unavailable)
        || (f.scratch.coverage == RemoteScratchCoverage::Unavailable
            && (f.scratch.samples != 0
                || f.scratch.sampled_peak_bytes.is_some()
                || f.scratch.end_snapshot_bytes.is_some()))
        || (f.scratch.coverage == RemoteScratchCoverage::Sampled
            && (f.scratch.samples == 0
                || f.scratch.sampled_peak_bytes.is_none()
                || f.scratch.end_snapshot_bytes.is_none()))
        || f.scratch
            .end_snapshot_bytes
            .is_some_and(|end| f.scratch.sampled_peak_bytes.is_none_or(|peak| end > peak))
        || !matches!((f.exit_code, f.term_signal), (Some(0..=255), None) | (None, Some(1..=64)))
    {
        return Err("remote-usage-facts-invalid".into());
    }
    Ok(())
}

pub struct RemoteUsageCollector {
    binding: RemoteProductionAttemptBinding,
    request_id: String,
    wall_limit: Duration,
    #[cfg(target_os = "linux")]
    cgroup: Option<CgroupProbe>,
    #[cfg(target_os = "linux")]
    scratch: Option<ScratchProbe>,
}

impl RemoteUsageCollector {
    pub fn begin(
        binding: &RemoteProductionAttemptBinding,
        request_id: &str,
        wall_limit: Duration,
    ) -> Result<Self, String> {
        validate_identity(binding, request_id)?;
        if wall_limit.is_zero() || wall_limit > Duration::from_secs(MAX_USAGE_WALL_SECS) {
            return Err("remote-usage-wall-limit-invalid".into());
        }
        Ok(Self {
            binding: binding.clone(),
            request_id: request_id.to_owned(),
            wall_limit,
            #[cfg(target_os = "linux")]
            cgroup: None,
            #[cfg(target_os = "linux")]
            scratch: None,
        })
    }
    pub fn with_cgroup_root(mut self, root: &std::path::Path, memory_limit_bytes: Option<u64>) -> Self {
        #[cfg(target_os = "linux")]
        {
            self.cgroup = CgroupProbe::create(root, &self.binding, &self.request_id, memory_limit_bytes).ok();
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (root, memory_limit_bytes);
        }
        self
    }
    pub fn with_scratch_root(mut self, root: &std::path::Path) -> Self {
        #[cfg(target_os = "linux")]
        {
            self.scratch = ScratchProbe::open(root).ok();
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = root;
        }
        self
    }

    /// Spawn the actual builder-side executor, never an SSH transport. Owns the
    /// wait: no other waiter may reap this child. stdin is preserved for a
    /// host-created regular input file; stdout/stderr are exclusively observed.
    #[cfg(target_os = "linux")]
    pub fn spawn(self, command: &mut Command) -> Result<RemoteUsageChild, String> {
        use std::os::fd::AsRawFd;
        use std::os::unix::process::CommandExt;
        let mut membership_status = None;
        if let Some(probe) = &self.cgroup {
            let procs = probe.procs.try_clone().map_err(|_| "remote-usage-cgroup-fd")?;
            let (reader, writer) = std::os::unix::net::UnixStream::pair().map_err(|_| "remote-usage-cgroup-status")?;
            membership_status = Some(reader);
            // SAFETY: only async-signal-safe writes to owned preopened fds.
            // The private status socket records the actual migration syscall,
            // not a caller-provided assertion. Unavailable delegation degrades.
            unsafe {
                command.pre_exec(move || {
                    let moved = libc::write(procs.as_raw_fd(), b"0".as_ptr().cast(), 1) == 1;
                    let status = if moved { b"1" } else { b"0" };
                    if libc::write(writer.as_raw_fd(), status.as_ptr().cast(), 1) == 1 {
                        Ok(())
                    } else {
                        Err(io::Error::last_os_error())
                    }
                });
            }
        }
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let started = Instant::now();
        let mut child = command.spawn().map_err(|e| format!("remote-usage-spawn: {e}"))?;
        let cgroup_migrated =
            membership_status.is_some_and(|reader| cgroup_membership_ack(reader, started, self.wall_limit));
        if let Err(error) = nonblocking(child.stdout.as_ref().expect("piped stdout"))
            .and_then(|()| nonblocking(child.stderr.as_ref().expect("piped stderr")))
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("remote-usage-pipe: {error}"));
        }
        Ok(RemoteUsageChild {
            collector: self,
            child,
            started,
            reaped: false,
            cgroup_migrated,
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub fn spawn(self, _command: &mut Command) -> Result<RemoteUsageChild, String> {
        Err("remote-usage-unsupported-platform".into())
    }
}

#[cfg(target_os = "linux")]
fn cgroup_membership_ack(mut reader: std::os::unix::net::UnixStream, started: Instant, wall_limit: Duration) -> bool {
    debug_assert!(!wall_limit.is_zero());
    debug_assert!(wall_limit.as_secs() <= MAX_USAGE_WALL_SECS);
    let remaining = wall_limit.saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return false;
    }
    if reader.set_read_timeout(Some(remaining)).is_err() {
        return false;
    }
    let mut status = [0_u8];
    // One byte needs no implicit read_exact retries after an interrupt.
    if !matches!(reader.read(&mut status), Ok(1)) {
        return false;
    }
    if started.elapsed() >= wall_limit {
        return false;
    }
    status == *b"1"
}

pub struct RemoteUsageChild {
    collector: RemoteUsageCollector,
    child: Child,
    started: Instant,
    reaped: bool,
    #[cfg(target_os = "linux")]
    cgroup_migrated: bool,
}

/// Captures are bounded independently from the exact byte counters. A caller
/// using stdout as a result protocol must reject `stdout_truncated()`.
pub struct RemoteUsageCompletion {
    facts: UsageFacts,
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl RemoteUsageCompletion {
    pub fn status(&self) -> ExitStatus {
        self.status
    }
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
    pub fn stdout_truncated(&self) -> bool {
        self.facts.stdout_bytes > self.stdout.len() as u64
    }
    /// Bind only after validating the actual executor result. A caller-supplied
    /// usage struct cannot enter this path: all facts already came from wait4.
    pub fn bind_result(mut self, result_digest_blake3: &str) -> Result<RemoteAuthenticatedUsage, String> {
        self.facts.result_digest_blake3 = result_digest_blake3.to_owned();
        validate_facts(&self.facts)?;
        Ok(RemoteAuthenticatedUsage { facts: self.facts })
    }
}

#[cfg(target_os = "linux")]
fn nonblocking(fd: &impl std::os::fd::AsRawFd) -> io::Result<()> {
    let raw = fd.as_raw_fd();
    // SAFETY: the pipe is owned and remains open for both fcntl calls.
    let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn drain(source: &mut impl Read, capture: &mut Vec<u8>, bytes: &mut u64, capture_limit: usize) -> Result<bool, String> {
    let mut buffer = [0; 16_384];
    // Fairness bounds each pass even if an untrusted child writes continuously.
    for _ in 0..16 {
        match source.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(count) => {
                *bytes = bytes
                    .checked_add(count as u64)
                    .filter(|n| *n <= MAX_USAGE_OUTPUT_BYTES)
                    .ok_or("remote-usage-output-overflow")?;
                let keep = count.min(capture_limit.saturating_sub(capture.len()));
                capture.extend_from_slice(&buffer[..keep]);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(e) => return Err(format!("remote-usage-read: {e}")),
        }
    }
    Ok(false)
}

impl RemoteUsageChild {
    #[cfg(target_os = "linux")]
    pub fn finish(mut self) -> Result<RemoteUsageCompletion, String> {
        use std::os::unix::process::ExitStatusExt;
        let pid = i32::try_from(self.child.id()).map_err(|_| "remote-usage-pid-invalid")?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut stdout_bytes = 0_u64;
        let mut stderr_bytes = 0_u64;
        let mut stdout_eof = false;
        let mut stderr_eof = false;
        let mut status = 0;
        // SAFETY: rusage consists entirely of C scalar fields, zero is valid.
        let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
        let mut next_scratch_sample = Instant::now();
        loop {
            if Instant::now() >= next_scratch_sample {
                if let Some(scratch) = &mut self.collector.scratch {
                    scratch.sample(false);
                }
                next_scratch_sample = Instant::now() + Duration::from_millis(100);
            }
            if !stdout_eof {
                stdout_eof = drain(
                    self.child.stdout.as_mut().expect("piped stdout"),
                    &mut stdout,
                    &mut stdout_bytes,
                    MAX_USAGE_STDOUT_BYTES,
                )?;
            }
            if !stderr_eof {
                stderr_eof = drain(
                    self.child.stderr.as_mut().expect("piped stderr"),
                    &mut stderr,
                    &mut stderr_bytes,
                    MAX_USAGE_STDERR_BYTES,
                )?;
            }
            if !self.reaped {
                // SAFETY: exact owned child PID; valid exclusive output pointers.
                let result = unsafe { libc::wait4(pid, &mut status, libc::WNOHANG, &mut usage) };
                if result == pid {
                    self.reaped = true;
                } else if result < 0 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::ECHILD) {
                        // An unexpected external waiter stole this child. Its
                        // PID may already be reused; never signal it on Drop.
                        self.reaped = true;
                    }
                    if error.kind() != io::ErrorKind::Interrupted {
                        return Err(format!("remote-usage-wait: {error}"));
                    }
                }
            }
            if self.reaped && stdout_eof && stderr_eof {
                break;
            }
            if self.started.elapsed() >= self.collector.wall_limit {
                return Err("remote-usage-incomplete-time-limit".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        fn micros(time: libc::timeval) -> Result<u64, String> {
            let seconds = u64::try_from(time.tv_sec).map_err(|_| "remote-usage-cpu-invalid")?;
            let micros = u64::try_from(time.tv_usec).map_err(|_| "remote-usage-cpu-invalid")?;
            if micros >= 1_000_000 {
                return Err("remote-usage-cpu-invalid".into());
            }
            seconds
                .checked_mul(1_000_000)
                .and_then(|n| n.checked_add(micros))
                .ok_or_else(|| "remote-usage-cpu-overflow".into())
        }
        let cpu_micros =
            micros(usage.ru_utime)?.checked_add(micros(usage.ru_stime)?).ok_or("remote-usage-cpu-overflow")?;
        // Linux reports ru_maxrss in KiB. This is a peak, not an integral.
        let peak_rss_bytes = u64::try_from(usage.ru_maxrss)
            .ok()
            .and_then(|n| n.checked_mul(1024))
            .ok_or("remote-usage-rss-invalid")?;
        let status = ExitStatus::from_raw(status);
        let cgroup_facts = self
            .collector
            .cgroup
            .as_ref()
            .and_then(|probe| probe.finish(self.started, self.collector.wall_limit).ok())
            .filter(|_| self.cgroup_migrated);
        let (completeness, oom, cgroup_peak_memory_bytes, cpu_time_millis) = match cgroup_facts {
            Some((oom, peak, cpu_ms)) => (RemoteUsageCompleteness::DedicatedCgroup, oom, peak, cpu_ms),
            None => {
                (RemoteUsageCompleteness::ProcessOnly, RemoteUsageOomEvidence::Unavailable, None, cpu_micros / 1000)
            }
        };
        let wall_time_millis =
            u64::try_from(self.started.elapsed().as_millis()).map_err(|_| "remote-usage-wall-overflow")?;
        let scratch = match &mut self.collector.scratch {
            Some(probe) => {
                probe.sample(true);
                probe.observation.clone()
            }
            None => RemoteScratchObservation::default(),
        };
        let facts = UsageFacts {
            schema: USAGE_DOMAIN.into(),
            binding: self.collector.binding.clone(),
            request_id: self.collector.request_id.clone(),
            result_digest_blake3: String::new(),
            child_pid: self.child.id(),
            cpu_time_millis,
            wall_time_millis,
            peak_rss_bytes,
            stdout_bytes,
            stderr_bytes,
            exit_code: status.code(),
            term_signal: status.signal(),
            oom,
            completeness,
            cgroup_peak_memory_bytes,
            scratch,
        };
        Ok(RemoteUsageCompletion {
            facts,
            status,
            stdout,
            stderr,
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub fn finish(self) -> Result<RemoteUsageCompletion, String> {
        Err("remote-usage-unsupported-platform".into())
    }
}

impl Drop for RemoteUsageChild {
    fn drop(&mut self) {
        if !self.reaped {
            // Kill while the owned PID has not been reaped, avoiding PID reuse.
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[cfg(target_os = "linux")]
fn openat_file(parent: &std::fs::File, name: &std::ffi::CStr, flags: i32) -> io::Result<std::fs::File> {
    use std::os::fd::AsRawFd;
    use std::os::fd::FromRawFd;
    // SAFETY: valid parent descriptor/C string; returned fd transfers ownership.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags | libc::O_CLOEXEC | libc::O_NOFOLLOW) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { std::fs::File::from_raw_fd(fd) })
    }
}

#[cfg(target_os = "linux")]
fn bounded_cgroup_read(directory: &std::fs::File, name: &std::ffi::CStr) -> io::Result<String> {
    let file = openat_file(directory, name, libc::O_RDONLY)?;
    let mut bytes = String::new();
    file.take(4097).read_to_string(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(io::Error::other("cgroup counter limit"));
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn named_counter(text: &str, key: &str) -> io::Result<u64> {
    let mut counter = None;
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words.next() == Some(key) {
            if counter.is_some() {
                return Err(io::Error::other("duplicate cgroup counter"));
            }
            counter = Some(
                words
                    .next()
                    .and_then(|n| n.parse().ok())
                    .ok_or_else(|| io::Error::other("invalid cgroup counter"))?,
            );
            if words.next().is_some() {
                return Err(io::Error::other("invalid cgroup counter"));
            }
        }
    }
    counter.ok_or_else(|| io::Error::other("missing cgroup counter"))
}

#[cfg(target_os = "linux")]
struct CgroupProbe {
    root: std::fs::File,
    directory: std::fs::File,
    procs: std::fs::File,
    name: std::ffi::CString,
}

#[cfg(target_os = "linux")]
impl CgroupProbe {
    fn create(
        root: &std::path::Path,
        binding: &RemoteProductionAttemptBinding,
        request_id: &str,
        memory_limit: Option<u64>,
    ) -> io::Result<Self> {
        use std::io::Write;
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::OpenOptionsExt;
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(root)?;
        let metadata = root.metadata()?;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o022 != 0 {
            return Err(io::Error::other("cgroup delegation ownership"));
        }
        let mut filesystem: libc::statfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstatfs(root.as_raw_fd(), &mut filesystem) } != 0 || filesystem.f_type as u64 != 0x6367_7270 {
            return Err(io::Error::other("not cgroup v2"));
        }
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let digest = failure_result_digest(binding, request_id);
        let name = std::ffi::CString::new(format!("mantle-{}-{}-{sequence}", &digest[..24], std::process::id()))
            .map_err(|_| io::Error::other("cgroup name"))?;
        if unsafe { libc::mkdirat(root.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let directory = match openat_file(&root, &name, libc::O_RDONLY | libc::O_DIRECTORY) {
            Ok(file) => file,
            Err(error) => {
                unsafe {
                    libc::unlinkat(root.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR);
                }
                return Err(error);
            }
        };
        let procs = match openat_file(&directory, c"cgroup.procs", libc::O_WRONLY) {
            Ok(file) => file,
            Err(error) => {
                unsafe {
                    libc::unlinkat(root.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR);
                }
                return Err(error);
            }
        };
        let probe = Self {
            root,
            directory,
            procs,
            name,
        };
        if unsafe { libc::fchmod(probe.directory.as_raw_fd(), 0o700) } != 0
            || !bounded_cgroup_read(&probe.directory, c"cgroup.procs")?.trim().is_empty()
            || named_counter(&bounded_cgroup_read(&probe.directory, c"memory.events")?, "oom_kill")? != 0
            || named_counter(&bounded_cgroup_read(&probe.directory, c"cpu.stat")?, "usage_usec")? != 0
        {
            return Err(io::Error::other("cgroup not fresh and exclusive"));
        }
        // Treat this per-attempt workload as one OOM victim so surviving
        // descendants cannot keep pipes/usage alive after the executor dies.
        let mut oom_group = openat_file(&probe.directory, c"memory.oom.group", libc::O_WRONLY)?;
        oom_group.write_all(b"1")?;
        if let Some(limit) = memory_limit {
            if limit == 0 {
                return Err(io::Error::other("zero cgroup memory limit"));
            }
            let mut maximum = openat_file(&probe.directory, c"memory.max", libc::O_WRONLY)?;
            maximum.write_all(limit.to_string().as_bytes())?;
        }
        Ok(probe)
    }
    fn finish(&self, started: Instant, wall_limit: Duration) -> io::Result<(RemoteUsageOomEvidence, Option<u64>, u64)> {
        debug_assert!(!wall_limit.is_zero());
        debug_assert!(wall_limit.as_secs() <= MAX_USAGE_WALL_SECS);
        // wait4 and pipe EOF can precede the kernel's final subtree removal.
        // Keep the original deadline; never promote a still-populated group.
        while named_counter(&bounded_cgroup_read(&self.directory, c"cgroup.events")?, "populated")? != 0 {
            if started.elapsed() >= wall_limit {
                return Err(io::Error::other("cgroup still populated at time limit"));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        // The event covers descendant subgroups; this also checks direct members.
        if !bounded_cgroup_read(&self.directory, c"cgroup.procs")?.trim().is_empty() {
            return Err(io::Error::other("cgroup still populated"));
        }
        let kills = named_counter(&bounded_cgroup_read(&self.directory, c"memory.events")?, "oom_kill")?;
        let cpu_us = named_counter(&bounded_cgroup_read(&self.directory, c"cpu.stat")?, "usage_usec")?;
        let peak = bounded_cgroup_read(&self.directory, c"memory.peak").ok().and_then(|n| n.trim().parse::<u64>().ok());
        Ok((
            if kills > 0 {
                RemoteUsageOomEvidence::DedicatedCgroupOom { oom_kill_delta: kills }
            } else {
                RemoteUsageOomEvidence::DedicatedCgroupNoOom
            },
            peak,
            cpu_us / 1000,
        ))
    }
}

#[cfg(target_os = "linux")]
impl Drop for CgroupProbe {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        // Only this fresh child group; never recursively remove/kill the root.
        unsafe {
            libc::unlinkat(self.root.as_raw_fd(), self.name.as_ptr(), libc::AT_REMOVEDIR);
        }
    }
}

#[cfg(target_os = "linux")]
struct ScratchProbe {
    root: std::fs::File,
    observation: RemoteScratchObservation,
}

#[cfg(target_os = "linux")]
impl ScratchProbe {
    fn open(path: &std::path::Path) -> io::Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        let root = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(path)?;
        Ok(Self {
            root,
            observation: RemoteScratchObservation::default(),
        })
    }
    fn sample(&mut self, final_sample: bool) {
        let mut seen = std::collections::BTreeSet::new();
        let mut visited = 0;
        let result = Self::walk(&self.root, 0, &mut seen, &mut visited);
        self.observation.samples = self.observation.samples.saturating_add(1);
        match result {
            Ok(bytes) => {
                self.observation.sampled_peak_bytes = Some(self.observation.sampled_peak_bytes.unwrap_or(0).max(bytes));
                if final_sample {
                    self.observation.end_snapshot_bytes = Some(bytes);
                }
                if self.observation.coverage != RemoteScratchCoverage::TraversalIncomplete {
                    self.observation.coverage = RemoteScratchCoverage::Sampled;
                }
            }
            Err(_) => self.observation.coverage = RemoteScratchCoverage::TraversalIncomplete,
        }
    }
    fn walk(
        directory: &std::fs::File,
        depth: usize,
        seen: &mut std::collections::BTreeSet<(u64, u64)>,
        visited: &mut usize,
    ) -> io::Result<u64> {
        use std::os::fd::AsRawFd;
        use std::os::fd::FromRawFd;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::MetadataExt;
        if depth > 32 {
            return Err(io::Error::other("scratch traversal depth"));
        }
        let entries = std::fs::read_dir(format!("/proc/self/fd/{}", directory.as_raw_fd()))?;
        let mut bytes = 0_u64;
        for entry in entries {
            *visited += 1;
            if *visited > 4096 {
                return Err(io::Error::other("scratch traversal entries"));
            }
            let entry = entry?;
            if entry.file_type()?.is_symlink() {
                continue;
            }
            let name = std::ffi::CString::new(entry.file_name().as_bytes())
                .map_err(|_| io::Error::other("scratch entry name"))?;
            #[repr(C)]
            struct OpenHow {
                flags: u64,
                mode: u64,
                resolve: u64,
            }
            // BENEATH | NO_SYMLINKS | NO_XDEV: never traverse a store bind mount
            // or another filesystem, including same-device bind mounts.
            let how = OpenHow {
                flags: (libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC) as u64,
                mode: 0,
                resolve: 0x08 | 0x04 | 0x01,
            };
            let fd = unsafe {
                libc::syscall(
                    libc::SYS_openat2,
                    directory.as_raw_fd(),
                    name.as_ptr(),
                    &how,
                    std::mem::size_of::<OpenHow>(),
                )
            };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            let file = unsafe { std::fs::File::from_raw_fd(fd as i32) };
            let metadata = file.metadata()?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if !seen.insert((metadata.dev(), metadata.ino())) {
                continue;
            }
            if metadata.is_dir() {
                bytes = bytes
                    .checked_add(Self::walk(&file, depth + 1, seen, visited)?)
                    .ok_or_else(|| io::Error::other("scratch bytes overflow"))?;
            } else if metadata.is_file() {
                bytes = bytes
                    .checked_add(
                        metadata
                            .blocks()
                            .checked_mul(512)
                            .ok_or_else(|| io::Error::other("scratch blocks overflow"))?,
                    )
                    .ok_or_else(|| io::Error::other("scratch bytes overflow"))?;
            }
        }
        Ok(bytes)
    }
}

#[cfg(all(test, target_os = "linux"))]
#[test]
fn cgroup_acknowledgement_requires_positive_migration_before_original_deadline() {
    use std::io::Write;
    use std::os::unix::net::UnixStream;

    for status in [b'0', b'1'] {
        let (reader, mut writer) = UnixStream::pair().unwrap();
        writer.write_all(&[status]).unwrap();
        assert_eq!(cgroup_membership_ack(reader, Instant::now(), Duration::from_secs(1)), status == b'1');
    }
    let (reader, mut writer) = UnixStream::pair().unwrap();
    writer.write_all(b"1").unwrap();
    let started = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    assert!(!cgroup_membership_ack(reader, started, Duration::from_secs(1)));

    // A pre-exec OOM can prevent the child from acknowledging its migration.
    // The parent's command still owns a writer: EOF alone cannot bound this wait.
    let (reader, _writer) = UnixStream::pair().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let waiting = std::thread::spawn(move || {
        sender.send(cgroup_membership_ack(reader, Instant::now(), Duration::from_millis(50))).unwrap();
    });
    assert!(!receiver.recv_timeout(Duration::from_secs(5)).expect("migration acknowledgement exceeded deadline"));
    waiting.join().unwrap();
}
