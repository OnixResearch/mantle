//! Opt-in Cargo wrapper observations. This module never supplies admission/cache decisions.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use mantle_rust_differential_core::Digest;
pub use mantle_rust_differential_core::waits::RunOutcome;
use mantle_rust_differential_core::waits::*;
use serde::Deserialize;
use serde::Serialize;
const MAX_CAPTURE_FRAMES: usize = 4096;
const MAX_CAPTURE_BYTES: usize = 2 * 1024 * 1024;
const MAX_CORRELATIONS: usize = 256;
const RESERVED_TERMINAL_FRAMES: usize = MAX_CORRELATIONS * 2;
const RESERVED_TERMINAL_BYTES: usize = RESERVED_TERMINAL_FRAMES * MAX_RECORD_BYTES;
const MAX_CARGO_FACTS: usize = 256;
const MAX_LINE_BYTES: usize = 64 * 1024;
const ENV_SOCKET: &str = "MANTLE_WAIT_SOCKET";
const ENV_CONTEXT: &str = "MANTLE_WAIT_CONTEXT";
const ENV_ORIGIN: &str = "MANTLE_WAIT_ORIGIN_NS";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaitContext {
    pub invocation: Digest,
    pub clock: Digest,
    pub toolchain: Digest,
    pub configuration: Digest,
    pub host: Digest,
    pub target: Digest,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    manifest: Option<Digest>,
    source_path: Option<Digest>,
    features: Vec<Digest>,
    opt_level: String,
    debuginfo: String,
    test: bool,
    explicit_target: Option<Digest>,
    proc_macro: bool,
    arguments: Digest,
    compiler: Digest,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FrameKind {
    Started,
    Error,
    Exited { outcome: RunOutcome },
    Reaped,
    SpawnFailed,
    Dropped,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    invocation: Digest,
    clock: Digest,
    token: Digest,
    sequence: u64,
    source_offset_ns: u64,
    binding: Option<Binding>,
    kind: FrameKind,
}
struct Received {
    frame: Frame,
    receipt_ns: u64,
}
struct Capture {
    records: Vec<Received>,
    dropped: u64,
    malformed: u64,
    bytes: usize,
}
#[derive(Clone)]
struct CargoFact {
    manifest: Digest,
    source_path: Digest,
    package: Digest,
    source: Digest,
    target: Digest,
    target_kind: Digest,
    features: Vec<Digest>,
    profile: Digest,
    opt_level: String,
    debuginfo: String,
    test: bool,
    host: bool,
    fresh: bool,
}

pub struct WaitCollector {
    root: PathBuf,
    wrapper: PathBuf,
    context: WaitContext,
    origin_ns: u64,
    socket_path: PathBuf,
    stop: Arc<AtomicBool>,
    receiver: Option<thread::JoinHandle<Capture>>,
    facts: Vec<CargoFact>,
    fact_overflow: u64,
    resources: ResourceReport,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagnosticOutcome {
    pub report: WaitReport,
    pub publication_error: Option<String>,
}

impl WaitCollector {
    /// Root must already be an exclusive invocation-owned directory, not a shared output root.
    pub fn start(root: &Path, wrapper: &Path, context: WaitContext) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "wait root must be private directory"));
        }
        let root = fs::canonicalize(root)?;
        let wrapper = fs::canonicalize(wrapper)?;
        let socket_path = root.join("wait.sock");
        let socket = UnixDatagram::bind(&socket_path)?;
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;
        socket.set_read_timeout(Some(Duration::from_millis(25)))?;
        let origin_ns = monotonic_ns()?;
        let stop = Arc::new(AtomicBool::new(false));
        let receiver_stop = stop.clone();
        let receiver_context = context.clone();
        let receiver = thread::spawn(move || capture(socket, receiver_stop, receiver_context, origin_ns));
        Ok(Self {
            root,
            wrapper,
            context,
            origin_ns,
            socket_path,
            stop,
            receiver: Some(receiver),
            facts: Vec::new(),
            fact_overflow: 0,
            resources: ResourceReport::unavailable(),
        })
    }
    pub fn offset_ns(&self) -> u64 {
        monotonic_ns().unwrap_or(self.origin_ns).saturating_sub(self.origin_ns)
    }
    /// Apply after env_clear. No process-global environment is mutated.
    pub fn configure(&self, command: &mut Command) {
        command
            .env("RUSTC_WRAPPER", &self.wrapper)
            .env(ENV_SOCKET, &self.socket_path)
            .env(ENV_ORIGIN, self.origin_ns.to_string());
        if let Ok(context) = serde_json::to_string(&self.context) {
            command.env(ENV_CONTEXT, context);
        }
    }
    /// Cargo records may arrive after completion. Their receipt is NOT metadata-ready timing.
    pub fn observe_cargo(&mut self, value: &serde_json::Value) {
        if value.get("reason").and_then(|v| v.as_str()) != Some("compiler-artifact") {
            return;
        }
        if self.facts.len() == MAX_CARGO_FACTS {
            self.fact_overflow = self.fact_overflow.saturating_add(1);
            return;
        }
        if let Some(fact) = cargo_fact(value) {
            self.facts.push(fact);
        } else {
            self.fact_overflow = self.fact_overflow.saturating_add(1);
        }
    }
    pub fn set_resources(&mut self, mut resources: ResourceReport, supervision_origin_ns: u64) {
        let Some(shift) = supervision_origin_ns.checked_sub(self.origin_ns) else {
            self.fact_overflow = self.fact_overflow.saturating_add(1);
            return;
        };
        for sample in &mut resources.samples {
            sample.offset_ns = sample.offset_ns.saturating_add(shift);
        }
        for gap in &mut resources.gaps {
            gap.start_ns = gap.start_ns.saturating_add(shift);
            gap.end_ns = gap.end_ns.map(|end| end.saturating_add(shift));
        }
        if shift > 0 {
            resources.gaps.insert(0, SampleGap {
                start_ns: 0,
                end_ns: Some(shift),
                reason: Unavailable::Unsampled,
            });
        }
        if resources.samples.len() <= MAX_CAPTURE_FRAMES
            && resources.gaps.len() <= MAX_CAPTURE_FRAMES
            && resources.interval_ns >= 100_000_000
            && matches!(resources.scope.as_str(), "process_tree" | "cgroup")
            && matches!(resources.metric.as_str(), "sum_rss_bytes" | "memory_current_bytes" | "memory_peak_bytes")
        {
            self.resources = resources;
        } else {
            self.fact_overflow = self.fact_overflow.saturating_add(1);
        }
    }
    /// Ordinary result is supplied, never changed, including when publication fails.
    pub fn finish(
        mut self,
        outcome: RunOutcome,
        supervision_origin_ns: u64,
        elapsed_ns: u64,
        last_reap_ns: Option<u64>,
    ) -> DiagnosticOutcome {
        let shift = supervision_origin_ns.checked_sub(self.origin_ns);
        if shift.is_none() {
            self.fact_overflow = self.fact_overflow.saturating_add(1);
        }
        let elapsed_ns = elapsed_ns.saturating_add(shift.unwrap_or(0));
        let last_reap_ns = shift.and_then(|shift| last_reap_ns.map(|end| end.saturating_add(shift)));
        self.stop.store(true, Ordering::Release);
        let capture = self.receiver.take().and_then(|handle| handle.join().ok()).unwrap_or(Capture {
            records: Vec::new(),
            dropped: 1,
            malformed: 1,
            bytes: 0,
        });
        let _ = fs::remove_file(&self.socket_path);
        let mut report =
            assemble(&self.context, capture, &self.facts, self.fact_overflow, outcome, elapsed_ns, last_reap_ns);
        report.resources = self.resources.clone();
        if report.resources.samples.is_empty() {
            report.resources.gaps.push(SampleGap {
                start_ns: 0,
                end_ns: Some(elapsed_ns),
                reason: Unavailable::Unsampled,
            });
        }
        let publication_error = publish(&self.root.join("rust-waits.json"), &report)
            .err()
            .map(|_| "diagnostic_publication_failed".to_string());
        report.sink_failed = publication_error.is_some();
        DiagnosticOutcome {
            report,
            publication_error,
        }
    }
}
impl Drop for WaitCollector {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.receiver.take() {
            let _ = handle.join();
        }
        let _ = fs::remove_file(&self.socket_path);
    }
}
fn capture(socket: UnixDatagram, stop: Arc<AtomicBool>, context: WaitContext, origin_ns: u64) -> Capture {
    let mut result = Capture {
        records: Vec::new(),
        dropped: 0,
        malformed: 0,
        bytes: 0,
    };
    let mut buffer = [0u8; MAX_RECORD_BYTES + 1];
    loop {
        match socket.recv(&mut buffer) {
            Ok(size) => {
                let receipt_ns = monotonic_ns().unwrap_or(origin_ns).saturating_sub(origin_ns);
                if size > MAX_RECORD_BYTES {
                    result.dropped = result.dropped.saturating_add(1);
                    continue;
                }
                match serde_json::from_slice::<Frame>(&buffer[..size]) {
                    Ok(frame) if frame.invocation == context.invocation && frame.clock == context.clock => {
                        if matches!(frame.kind, FrameKind::Dropped) {
                            result.dropped = result.dropped.saturating_add(1);
                            continue;
                        }
                        if !valid_binding(frame.binding.as_ref()) {
                            result.malformed = result.malformed.saturating_add(1);
                            continue;
                        }
                        let terminal =
                            matches!(frame.kind, FrameKind::Exited { .. } | FrameKind::Reaped | FrameKind::SpawnFailed);
                        let record_limit = if terminal {
                            MAX_CAPTURE_FRAMES
                        } else {
                            MAX_CAPTURE_FRAMES - RESERVED_TERMINAL_FRAMES
                        };
                        let byte_limit = if terminal {
                            MAX_CAPTURE_BYTES
                        } else {
                            MAX_CAPTURE_BYTES - RESERVED_TERMINAL_BYTES
                        };
                        if result.records.len() >= record_limit || result.bytes.saturating_add(size) > byte_limit {
                            result.dropped = result.dropped.saturating_add(1);
                            continue;
                        }
                        result.bytes += size;
                        result.records.push(Received { frame, receipt_ns });
                    }
                    _ => result.malformed = result.malformed.saturating_add(1),
                }
            }
            Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {
                if stop.load(Ordering::Acquire) {
                    break;
                }
            }
            Err(_) => {
                result.dropped = result.dropped.saturating_add(1);
                break;
            }
        }
    }
    result
}
fn valid_binding(binding: Option<&Binding>) -> bool {
    binding.is_none_or(|b| {
        b.features.len() <= MAX_PREREQUISITES
            && matches!(b.opt_level.as_str(), "0" | "1" | "2" | "3" | "s" | "z")
            && matches!(
                b.debuginfo.as_str(),
                "0" | "1" | "2" | "line-tables-only" | "line-directives-only" | "limited" | "full" | "none"
            )
    })
}
fn digest_json<T: Serialize>(domain: &str, value: &T) -> Digest {
    // Closed typed values used here have infallible JSON serialization.
    let bytes = serde_json::to_vec(&(domain, value)).unwrap_or_default();
    Digest::from_slice(&bytes)
}
fn path_digest(path: &Path) -> Digest {
    Digest::from_slice(path.as_os_str().as_bytes())
}
fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    fs::canonicalize(joined)
}
fn cargo_fact(value: &serde_json::Value) -> Option<CargoFact> {
    let manifest = value.get("manifest_path")?.as_str()?;
    let package = value.get("package_id")?.as_str()?;
    let target = value.get("target")?;
    let source_path = target.get("src_path")?.as_str()?;
    let kind = target.get("kind")?.as_array()?;
    let profile = value.get("profile")?;
    let features = value.get("features")?.as_array()?;
    if manifest.len() > 4096
        || source_path.len() > 4096
        || package.len() > 4096
        || kind.len() > 8
        || features.len() > MAX_PREREQUISITES
    {
        return None;
    }
    let mut feature_ids = Vec::with_capacity(features.len());
    for feature in features {
        let name = feature.as_str()?;
        if name.len() > 256 {
            return None;
        }
        feature_ids.push(Digest::from_slice(name.as_bytes()));
    }
    feature_ids.sort();
    feature_ids.dedup();
    let opt_level = profile.get("opt_level")?.as_str()?.to_string();
    let debuginfo = match profile.get("debuginfo")? {
        serde_json::Value::Null => "0".into(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) if s.len() < 32 => s.clone(),
        _ => return None,
    };
    if opt_level.len() > 8 {
        return None;
    }
    Some(CargoFact {
        manifest: path_digest(Path::new(manifest)),
        source_path: path_digest(Path::new(source_path)),
        package: Digest::from_slice(package.as_bytes()),
        source: digest_json("cargo-package-source-v1", &package),
        target: digest_json("cargo-target-v1", target),
        target_kind: digest_json("cargo-target-kind-v1", kind),
        features: feature_ids,
        profile: digest_json("cargo-profile-v1", profile),
        opt_level,
        debuginfo,
        test: profile.get("test")?.as_bool()?,
        host: kind.iter().any(|k| matches!(k.as_str(), Some("proc-macro" | "custom-build"))),
        fresh: value.get("fresh")?.as_bool()?,
    })
}
fn matches_binding(binding: &Binding, fact: &CargoFact) -> bool {
    !fact.fresh
        && binding.manifest.as_ref() == Some(&fact.manifest)
        && binding.source_path.as_ref() == Some(&fact.source_path)
        && binding.features == fact.features
        && binding.opt_level == fact.opt_level
        && binding.debuginfo == fact.debuginfo
        && binding.test == fact.test
}
fn assemble(
    context: &WaitContext,
    capture: Capture,
    facts: &[CargoFact],
    overflow: u64,
    outcome: RunOutcome,
    elapsed_ns: u64,
    last_reap_ns: Option<u64>,
) -> WaitReport {
    let mut timeline = Timeline {
        invocation: context.invocation.clone(),
        clock: context.clock.clone(),
        units: Vec::new(),
        attempts: Vec::new(),
        events: Vec::new(),
        dropped_records: capture.dropped.saturating_add(capture.malformed).saturating_add(overflow),
        complete: capture.dropped == 0 && capture.malformed == 0 && overflow == 0,
    };
    let mut bindings: BTreeMap<&Digest, &Binding> = BTreeMap::new();
    let mut tokens = BTreeSet::new();
    for received in &capture.records {
        if tokens.len() >= MAX_CORRELATIONS && !tokens.contains(&received.frame.token) {
            timeline.complete = false;
            timeline.dropped_records = timeline.dropped_records.saturating_add(1);
            continue;
        }
        tokens.insert(&received.frame.token);
        if let Some(binding) = &received.frame.binding {
            bindings.entry(&received.frame.token).or_insert(binding);
        }
    }
    let mut ids = BTreeMap::new();
    for token in tokens {
        let binding = bindings.get(token).copied();
        let matches: Vec<_> =
            binding.map(|b| facts.iter().filter(|f| matches_binding(b, f)).collect()).unwrap_or_default();
        // Two launches that fit the same artifact fact remain ambiguous; never use nearest time.
        let unique = matches.len() == 1 && bindings.values().filter(|b| matches_binding(b, matches[0])).count() == 1;
        let fact = if unique { Some(matches[0]) } else { None };
        let role = match (binding, fact) {
            (_, Some(f)) if f.host => ProcessRole::HostCompiler,
            (Some(b), _) if b.proc_macro => ProcessRole::HostCompiler,
            (Some(b), _) if b.source_path.is_some() => ProcessRole::TargetCompiler,
            _ => ProcessRole::ToolProbe,
        };
        let unit = match (binding, fact) {
            (Some(b), Some(f)) => {
                let host = context.host.clone();
                let triple = if f.host { host.clone() } else { context.target.clone() };
                let facts = UnitIdentity {
                    package: f.package.clone(),
                    source: digest_json("wait-package-content-v1", &(&f.source, &context.configuration)),
                    target: f.target.clone(),
                    target_kind: f.target_kind.clone(),
                    host,
                    target_triple: triple,
                    role: role.clone(),
                    features: b.features.clone(),
                    profile: f.profile.clone(),
                    mode: context.configuration.clone(),
                    compiler: b.compiler.clone(),
                    arguments: b.arguments.clone(),
                    inputs: context.configuration.clone(),
                    dependencies: Vec::new(),
                };
                match facts.identity() {
                    Ok(id) => {
                        timeline.units.push(Unit { id: id.clone(), facts });
                        Some(id)
                    }
                    Err(_) => None,
                }
            }
            _ => None,
        };
        let resolved_process = digest_json(
            "wait-resolved-wrapper-process-v1",
            &(token, binding.map(|b| (&b.arguments, &b.compiler)), &context.configuration),
        );
        let unmapped = if unit.is_some() {
            None
        } else if matches.is_empty() {
            Some(Unavailable::Unmapped)
        } else {
            Some(Unavailable::AmbiguousAssociation)
        };
        let mut attempt = Attempt {
            id: token.clone(),
            invocation: context.invocation.clone(),
            clock: context.clock.clone(),
            unit,
            role,
            number: 1,
            resolved_process,
            prior: None,
            unmapped,
        };
        attempt.id = attempt.identity();
        ids.insert(token.clone(), attempt.id.clone());
        timeline.attempts.push(attempt);
    }
    let mut sequence = 0u64;
    let mut started = BTreeSet::new();
    let mut exited = BTreeSet::new();
    for received in &capture.records {
        let frame = &received.frame;
        let Some(id) = ids.get(&frame.token) else {
            continue;
        };
        let kind = match &frame.kind {
            FrameKind::Started => {
                started.insert(id.clone());
                EventKind::Started
            }
            FrameKind::Error if started.contains(id) && !exited.contains(id) => EventKind::ErrorObserved,
            FrameKind::Exited { outcome } if started.contains(id) => {
                exited.insert(id.clone());
                EventKind::Exited {
                    outcome: outcome.clone(),
                }
            }
            FrameKind::Reaped if exited.contains(id) => EventKind::Reaped,
            FrameKind::SpawnFailed => {
                timeline.complete = false;
                continue;
            }
            _ => {
                timeline.complete = false;
                continue;
            }
        };
        sequence += 1;
        timeline.events.push(Event {
            reference: EventRef {
                invocation: context.invocation.clone(),
                clock: context.clock.clone(),
                sequence,
            },
            offset_ns: received.receipt_ns,
            attempt: id.clone(),
            source_sequence: Some(frame.sequence),
            kind,
        });
    }
    if started != exited {
        timeline.complete = false;
    }
    let first_error = timeline.events.iter().find(|e| matches!(e.kind, EventKind::ErrorObserved)).map(|e| e.offset_ns);
    let first_error_ns = first_error.map_or(Metric::Unavailable(Unavailable::MissingEndpoint), Metric::Observed);
    let error_drain_ns = match (first_error, last_reap_ns) {
        (Some(start), Some(end)) if end >= start => Metric::Observed(end - start),
        _ => Metric::Unavailable(Unavailable::Censored),
    };
    let (intervals, reduction_error) = match reduce(&timeline) {
        Ok(v) => (v, None),
        Err(error) => {
            timeline.complete = false;
            (Vec::new(), Some(format!("{error:?}")))
        }
    };
    WaitReport {
        schema: SCHEMA.to_string(),
        timeline,
        intervals,
        ordinary_outcome: outcome,
        elapsed_ns,
        last_reap_ns,
        first_error_ns,
        error_drain_ns,
        resources: ResourceReport::unavailable(),
        unavailable_phases: vec![Unavailable::UnsupportedProtocol],
        reduction_error,
        sink_failed: false,
    }
}
struct BoundedWriter {
    bytes: Vec<u8>,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > MAX_SIDECAR_BYTES {
            return Err(io::Error::other("wait sidecar bound"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn publish(path: &Path, report: &WaitReport) -> io::Result<()> {
    let mut writer = BoundedWriter { bytes: Vec::new() };
    serde_json::to_writer(&mut writer, report)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    file.write_all(&writer.bytes)?;
    file.sync_all()
}
pub(crate) fn monotonic_ns() -> io::Result<u64> {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: valid writable timespec; CLOCK_MONOTONIC is invocation-local host time only.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let seconds = u64::try_from(time.tv_sec).map_err(|_| io::Error::other("negative monotonic clock"))?;
    let nanos = u64::try_from(time.tv_nsec).map_err(|_| io::Error::other("negative monotonic clock"))?;
    seconds
        .checked_mul(1_000_000_000)
        .and_then(|v| v.checked_add(nanos))
        .ok_or_else(|| io::Error::other("monotonic overflow"))
}

/// Cargo RUSTC_WRAPPER entry. Compiler diagnostics are forwarded byte-for-byte, not retained here.
pub fn wrapper_main() -> i32 {
    let mut arguments = std::env::args_os().skip(1);
    let Some(compiler) = arguments.next() else {
        return 127;
    };
    let args: Vec<OsString> = arguments.take(4097).collect();
    if args.len() > 4096 {
        // Observation bounds must not truncate or replace the ordinary compiler command.
        use std::os::unix::process::CommandExt;
        let error = Command::new(&compiler).args(std::env::args_os().skip(2)).exec();
        let _ = error;
        return 127;
    }
    let observer = WrapperObserver::from_environment(&compiler, &args);
    let mut command = Command::new(&compiler);
    command.args(&args).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            if let Some(mut observer) = observer {
                observer.emit(FrameKind::SpawnFailed, false);
            }
            return 127;
        }
    };
    let mut observer = observer;
    if let Some(o) = observer.as_mut() {
        o.emit(FrameKind::Started, true);
    }
    let stderr = child.stderr.take();
    let forwarding = thread::spawn(move || {
        let mut observer = observer;
        if let Some(mut source) = stderr {
            let mut buffer = [0u8; 8192];
            let mut line = Vec::new();
            let mut oversized = false;
            let mut output = io::stderr().lock();
            loop {
                let size = match source.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(size) => size,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                };
                // Output failure is not permission to leave the compiler blocked on its pipe.
                let _ = output.write_all(&buffer[..size]);
                for byte in &buffer[..size] {
                    if *byte == b'\n' {
                        if !oversized
                            && is_error(&line)
                            && let Some(o) = observer.as_mut()
                        {
                            o.emit(FrameKind::Error, false);
                        }
                        line.clear();
                        oversized = false;
                    } else if !oversized {
                        if line.len() < MAX_LINE_BYTES {
                            line.push(*byte);
                        } else {
                            line.clear();
                            oversized = true;
                        }
                    }
                }
            }
            if !oversized
                && !line.is_empty()
                && is_error(&line)
                && let Some(o) = observer.as_mut()
            {
                o.emit(FrameKind::Error, false);
            }
        }
        observer
    });
    let status = child.wait();
    let mut observer = forwarding.join().ok().flatten();
    let outcome = match &status {
        Ok(s) if s.success() => RunOutcome::Success,
        Ok(s) if s.code().is_none() => RunOutcome::Crash,
        Ok(_) => RunOutcome::Rejected,
        Err(_) => RunOutcome::Infrastructure,
    };
    if let Some(o) = observer.as_mut() {
        o.emit(FrameKind::Exited { outcome }, false);
        if status.is_ok() {
            o.emit(FrameKind::Reaped, false);
        }
    }
    match status {
        Ok(s) => {
            use std::os::unix::process::ExitStatusExt;
            if let Some(signal) = s.signal() {
                // SAFETY: restoring and raising the observed child signal preserves wrapper failure semantics.
                unsafe {
                    libc::signal(signal, libc::SIG_DFL);
                    libc::raise(signal);
                }
                return 128 + signal;
            }
            s.code().unwrap_or(127)
        }
        Err(_) => 127,
    }
}
fn is_error(bytes: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(bytes).ok().is_some_and(|v| {
        v.get("$message_type").and_then(|v| v.as_str()) == Some("diagnostic")
            && v.get("level").and_then(|v| v.as_str()) == Some("error")
    })
}
struct WrapperObserver {
    socket: UnixDatagram,
    context: WaitContext,
    origin_ns: u64,
    token: Digest,
    sequence: u64,
    binding: Binding,
    dropped: bool,
}
impl WrapperObserver {
    fn from_environment(compiler: &OsStr, args: &[OsString]) -> Option<Self> {
        let context = std::env::var(ENV_CONTEXT).ok()?;
        if context.len() > MAX_RECORD_BYTES {
            return None;
        }
        let context: WaitContext = serde_json::from_str(&context).ok()?;
        let origin_ns = std::env::var(ENV_ORIGIN).ok()?.parse().ok()?;
        let socket = UnixDatagram::unbound().ok()?;
        socket.connect(std::env::var_os(ENV_SOCKET)?).ok()?;
        socket.set_nonblocking(true).ok()?;
        let mut nonce = [0u8; 32];
        File::open("/dev/urandom").ok()?.read_exact(&mut nonce).ok()?;
        let token = digest_json("wait-wrapper-launch-v1", &(&context.invocation, nonce));
        let binding = wrapper_binding(compiler, args, &context)?;
        Some(Self {
            socket,
            context,
            origin_ns,
            token,
            sequence: 0,
            binding,
            dropped: false,
        })
    }
    fn emit(&mut self, kind: FrameKind, include_binding: bool) {
        self.sequence = self.sequence.saturating_add(1);
        let frame = Frame {
            invocation: self.context.invocation.clone(),
            clock: self.context.clock.clone(),
            token: self.token.clone(),
            sequence: self.sequence,
            source_offset_ns: monotonic_ns().unwrap_or(self.origin_ns).saturating_sub(self.origin_ns),
            binding: include_binding.then(|| self.binding.clone()),
            kind,
        };
        let Ok(bytes) = serde_json::to_vec(&frame) else {
            self.dropped = true;
            return;
        };
        if bytes.len() > MAX_RECORD_BYTES || self.socket.send(&bytes).is_err() {
            self.dropped = true;
        }
        // At most one additional bounded nonblocking loss marker per ordinary emission.
        if self.dropped {
            let marker = Frame {
                binding: None,
                kind: FrameKind::Dropped,
                ..frame
            };
            if let Ok(bytes) = serde_json::to_vec(&marker)
                && self.socket.send(&bytes).is_ok()
            {
                self.dropped = false;
            }
        }
    }
}
fn wrapper_binding(compiler: &OsStr, args: &[OsString], context: &WaitContext) -> Option<Binding> {
    if args.len() > 4096 {
        return None;
    }
    let mut argument_hasher = blake3::Hasher::new();
    argument_hasher.update(b"mantle-rust-wait-raw-os-argv-v1\0");
    let mut source_path = None;
    let mut features = Vec::new();
    let mut opt_level = "0".to_string();
    let mut debuginfo = "0".to_string();
    let mut explicit_target = None;
    let mut proc_macro = false;
    let mut test = false;
    let mut bytes = 0usize;
    for (index, arg) in args.iter().enumerate() {
        bytes = bytes.checked_add(arg.len())?;
        if bytes > 1024 * 1024 {
            return None;
        }
        argument_hasher.update(&(arg.len() as u64).to_le_bytes());
        argument_hasher.update(arg.as_bytes());
        if arg.as_bytes() == b"--test" {
            test = true;
        }
        if arg.as_bytes() == b"--target" {
            explicit_target = args.get(index + 1).map(|v| Digest::from_slice(v.as_bytes()));
        }
        if arg.as_bytes() == b"--crate-type" {
            proc_macro |= args.get(index + 1).is_some_and(|v| v.as_bytes() == b"proc-macro");
        }
        if let Some(text) = arg.to_str() {
            if text == "--crate-type=proc-macro" {
                proc_macro = true;
            }
            if let Some(value) = text.strip_prefix("--target=") {
                explicit_target = Some(Digest::from_slice(value.as_bytes()));
            }
            let codegen = if text == "-C" {
                args.get(index + 1).and_then(|v| v.to_str())
            } else {
                text.strip_prefix("-C")
            };
            if let Some(value) = codegen.and_then(|v| v.strip_prefix("opt-level=")) {
                opt_level = value.to_string();
            }
            if let Some(value) = codegen.and_then(|v| v.strip_prefix("debuginfo=")) {
                debuginfo = value.to_string();
            }
            if text == "--cfg"
                && let Some(value) = args
                    .get(index + 1)
                    .and_then(|v| v.to_str())
                    .and_then(|v| v.strip_prefix("feature=\""))
                    .and_then(|v| v.strip_suffix('"'))
            {
                features.push(Digest::from_slice(value.as_bytes()));
            }
        }
        if !arg.as_bytes().starts_with(b"-") && Path::new(arg).extension() == Some(OsStr::new("rs")) {
            let path = absolute_path(Path::new(arg)).ok()?;
            if source_path.replace(path_digest(&path)).is_some() {
                return None;
            }
        }
    }
    if features.len() > MAX_PREREQUISITES {
        return None;
    }
    features.sort();
    features.dedup();
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .and_then(|dir| absolute_path(&PathBuf::from(dir).join("Cargo.toml")).ok())
        .map(|p| path_digest(&p));
    let compiler_path = absolute_path(Path::new(compiler)).ok()?;
    let mut file = File::open(&compiler_path).ok()?;
    let mut compiler_hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536];
    let mut total = 0u64;
    loop {
        let size = file.read(&mut buffer).ok()?;
        if size == 0 {
            break;
        }
        total = total.checked_add(size as u64)?;
        if total > 512 * 1024 * 1024 {
            return None;
        }
        compiler_hasher.update(&buffer[..size]);
    }
    let compiler =
        digest_json("wait-compiler-binary-toolchain-v1", &(&context.toolchain, compiler_hasher.finalize().as_bytes()));
    Some(Binding {
        manifest,
        source_path,
        features,
        opt_level,
        debuginfo,
        test,
        explicit_target,
        proc_macro,
        arguments: Digest::from_slice(argument_hasher.finalize().as_bytes()),
        compiler,
    })
}

/// Linux /proc process-tree sampler. Samples are lower bounds, never wait/phase evidence.
pub struct ProcessTreeSampler {
    root_pid: u32,
    report: ResourceReport,
    cpu: BTreeMap<(u32, u64), (u64, u64)>,
    last_ns: Option<u64>,
}
impl ProcessTreeSampler {
    pub fn new(root_pid: u32) -> Self {
        Self {
            root_pid,
            report: ResourceReport::unavailable(),
            cpu: BTreeMap::new(),
            last_ns: None,
        }
    }
    pub fn sample(&mut self, offset_ns: u64) {
        if self.last_ns.is_some_and(|last| offset_ns.saturating_sub(last) < self.report.interval_ns) {
            return;
        }
        if self.report.samples.len() >= MAX_CAPTURE_FRAMES {
            self.gap(offset_ns);
            return;
        }
        if let Some(last) = self.last_ns
            && offset_ns.saturating_sub(last) > self.report.interval_ns.saturating_mul(2)
        {
            self.report.gaps.push(SampleGap {
                start_ns: last,
                end_ns: Some(offset_ns),
                reason: Unavailable::Unsampled,
            });
        }
        self.last_ns = Some(offset_ns);
        let result = proc_tree(self.root_pid);
        let Ok(processes) = result else {
            self.gap(offset_ns);
            return;
        };
        let mut rss = 0u64;
        for p in &processes {
            if self.cpu.len() >= MAX_UNITS && !self.cpu.contains_key(&(p.pid, p.start)) {
                self.gap(offset_ns);
                return;
            }
            rss = rss.saturating_add(p.rss);
            let cpu = self.cpu.entry((p.pid, p.start)).or_default();
            cpu.0 = cpu.0.max(p.user);
            cpu.1 = cpu.1.max(p.system);
        }
        let (user, system) =
            self.cpu.values().fold((0u64, 0u64), |(u, s), (a, b)| (u.saturating_add(*a), s.saturating_add(*b)));
        self.report.samples.push(ResourceSample {
            offset_ns,
            rss_bytes: rss,
            cpu_user_ns: user,
            cpu_system_ns: system,
            processes: processes.len() as u32,
        });
    }
    fn gap(&mut self, offset_ns: u64) {
        if self.report.gaps.len() < MAX_CAPTURE_FRAMES {
            self.report.gaps.push(SampleGap {
                start_ns: offset_ns,
                end_ns: None,
                reason: Unavailable::Unsampled,
            });
        }
    }
    pub fn finish(mut self, elapsed_ns: u64) -> ResourceReport {
        if let Some(first) = self.report.samples.first()
            && first.offset_ns > 0
        {
            self.report.gaps.push(SampleGap {
                start_ns: 0,
                end_ns: Some(first.offset_ns),
                reason: Unavailable::Unsampled,
            });
        }
        if let Some(last) = self.report.samples.last() {
            if last.offset_ns < elapsed_ns {
                self.report.gaps.push(SampleGap {
                    start_ns: last.offset_ns,
                    end_ns: Some(elapsed_ns),
                    reason: Unavailable::Unsampled,
                });
            }
            self.report.cpu_user_ns = Metric::Observed(last.cpu_user_ns);
            self.report.cpu_system_ns = Metric::Observed(last.cpu_system_ns);
            self.report.peak_bytes = self
                .report
                .samples
                .iter()
                .map(|s| s.rss_bytes)
                .max()
                .map_or(Metric::Unavailable(Unavailable::Unsampled), Metric::Observed);
        }
        self.report
    }
}
struct Proc {
    pid: u32,
    parent: u32,
    start: u64,
    rss: u64,
    user: u64,
    system: u64,
}
fn proc_tree(root: u32) -> io::Result<Vec<Proc>> {
    // SAFETY: sysconf with constant selectors has no pointer preconditions.
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if ticks <= 0 || page <= 0 {
        return Err(io::Error::other("resource clock unavailable"));
    }
    let mut all = Vec::new();
    let mut entries = 0u32;
    for entry in fs::read_dir("/proc")? {
        entries = entries.saturating_add(1);
        if entries > 32768 {
            return Err(io::Error::other("process table bound"));
        }
        let entry = entry?;
        let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let mut file = match File::open(entry.path().join("stat")) {
            Ok(f) => f,
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied) => continue,
            Err(e) => return Err(e),
        };
        let mut text = String::new();
        Read::by_ref(&mut file).take(8193).read_to_string(&mut text)?;
        if text.len() > 8192 {
            return Err(io::Error::other("process record bound"));
        }
        let Some(end) = text.rfind(')') else {
            continue;
        };
        let fields: Vec<_> = text[end + 1..].split_whitespace().collect();
        if fields.len() < 22 {
            continue;
        }
        let parse = |index: usize| fields[index].parse::<u64>().ok();
        let (Some(parent), Some(user), Some(system), Some(start), Some(rss)) =
            (parse(1), parse(11), parse(12), parse(19), parse(21))
        else {
            continue;
        };
        all.push(Proc {
            pid,
            parent: parent as u32,
            start,
            rss: rss.saturating_mul(page as u64),
            user: user.saturating_mul(1_000_000_000) / ticks as u64,
            system: system.saturating_mul(1_000_000_000) / ticks as u64,
        });
    }
    let mut selected = BTreeSet::new();
    selected.insert(root);
    for _ in 0..MAX_UNITS {
        let old = selected.len();
        for p in &all {
            if selected.contains(&p.parent) {
                selected.insert(p.pid);
            }
        }
        if selected.len() > MAX_UNITS {
            return Err(io::Error::other("process tree bound"));
        }
        if old == selected.len() {
            break;
        }
    }
    let result: Vec<_> = all.into_iter().filter(|p| selected.contains(&p.pid)).collect();
    if !result.iter().any(|p| p.pid == root) {
        return Err(io::Error::other("process root absent"));
    }
    Ok(result)
}

/// Native local/session adapter collector. Caller owns exact admitted identities and effects.
/// Only explicit transition methods sample the invocation clock; no retrospective phases.
pub struct NativeWaitCollector {
    root: PathBuf,
    origin_ns: u64,
    timeline: Timeline,
    bytes: usize,
    tree_reap_ns: Option<u64>,
}
impl NativeWaitCollector {
    pub fn start(root: &Path, invocation: Digest, clock: Digest) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "native wait root must be private"));
        }
        Ok(Self {
            root: fs::canonicalize(root)?,
            origin_ns: monotonic_ns()?,
            timeline: Timeline {
                invocation,
                clock,
                units: Vec::new(),
                attempts: Vec::new(),
                events: Vec::new(),
                dropped_records: 0,
                complete: true,
            },
            bytes: 0,
            tree_reap_ns: None,
        })
    }
    pub fn offset_ns(&self) -> u64 {
        monotonic_ns().unwrap_or(self.origin_ns).saturating_sub(self.origin_ns)
    }
    pub fn register_unit(&mut self, facts: UnitIdentity) -> Result<Digest, WaitError> {
        let id = facts.identity()?;
        if self.timeline.units.iter().any(|u| u.id == id) {
            return Ok(id);
        }
        if self.timeline.units.len() >= MAX_CORRELATIONS {
            self.drop_record();
            return Err(WaitError::Bounds);
        }
        self.timeline.units.push(Unit { id: id.clone(), facts });
        Ok(id)
    }
    pub fn register_attempt(&mut self, mut attempt: Attempt) -> Result<Digest, WaitError> {
        if attempt.invocation != self.timeline.invocation || attempt.clock != self.timeline.clock {
            return Err(WaitError::CrossClock);
        }
        if self.timeline.attempts.len() >= MAX_CORRELATIONS {
            self.drop_record();
            return Err(WaitError::Bounds);
        }
        attempt.id = attempt.identity();
        if self.timeline.attempts.iter().any(|a| a.id == attempt.id) {
            return Err(WaitError::Duplicate);
        }
        let id = attempt.id.clone();
        self.timeline.attempts.push(attempt);
        Ok(id)
    }
    /// Emit immediately after real spawn; spawn failures intentionally have no Started.
    pub fn started(&mut self, attempt: &Digest) {
        self.observe(attempt, EventKind::Started);
    }
    pub fn exited(&mut self, attempt: &Digest, outcome: RunOutcome) {
        self.observe(attempt, EventKind::Exited { outcome });
    }
    pub fn reaped(&mut self, attempt: &Digest) {
        self.observe(attempt, EventKind::Reaped);
    }
    pub fn cache_restored(&mut self, attempt: &Digest, receipt: Digest) {
        self.observe(attempt, EventKind::CacheRestored { receipt });
    }
    /// Only call on a live, attempt-bound structured error receipt, not completion capture.
    pub fn error_observed(&mut self, attempt: &Digest) {
        self.observe(attempt, EventKind::ErrorObserved);
    }
    /// Typed scheduler/protocol observations only where the adapter actually has this boundary.
    pub fn observe(&mut self, attempt: &Digest, kind: EventKind) {
        let terminal = matches!(kind, EventKind::Exited { .. } | EventKind::Reaped);
        let limit = if terminal {
            MAX_CAPTURE_FRAMES
        } else {
            MAX_CAPTURE_FRAMES - RESERVED_TERMINAL_FRAMES
        };
        let sequence = self.timeline.events.len() as u64 + 1;
        let event = Event {
            reference: EventRef {
                invocation: self.timeline.invocation.clone(),
                clock: self.timeline.clock.clone(),
                sequence,
            },
            offset_ns: self.offset_ns(),
            attempt: attempt.clone(),
            source_sequence: None,
            kind,
        };
        let Ok(bytes) = serde_json::to_vec(&event) else {
            self.drop_record();
            return;
        };
        if self.timeline.events.len() >= limit
            || bytes.len() > MAX_RECORD_BYTES
            || self.bytes.saturating_add(bytes.len()) > MAX_CAPTURE_BYTES
        {
            self.drop_record();
            return;
        }
        self.bytes += bytes.len();
        self.timeline.events.push(event);
    }
    pub fn tree_reaped(&mut self) {
        self.tree_reap_ns = Some(self.offset_ns());
    }
    fn drop_record(&mut self) {
        self.timeline.complete = false;
        self.timeline.dropped_records = self.timeline.dropped_records.saturating_add(1);
    }
    pub fn finish(mut self, outcome: RunOutcome) -> DiagnosticOutcome {
        let elapsed_ns = self.offset_ns();
        let (intervals, reduction_error) = match reduce(&self.timeline) {
            Ok(v) => (v, None),
            Err(e) => {
                self.timeline.complete = false;
                (Vec::new(), Some(format!("{e:?}")))
            }
        };
        let first = self
            .timeline
            .events
            .iter()
            .find(|e| matches!(e.kind, EventKind::ErrorObserved))
            .map(|e| e.offset_ns);
        let drain = match (first, self.tree_reap_ns) {
            (Some(a), Some(b)) if b >= a => Metric::Observed(b - a),
            _ => Metric::Unavailable(Unavailable::Censored),
        };
        let mut resources = ResourceReport::unavailable();
        resources.gaps.push(SampleGap {
            start_ns: 0,
            end_ns: Some(elapsed_ns),
            reason: Unavailable::Unsampled,
        });
        let mut report = WaitReport {
            schema: SCHEMA.into(),
            timeline: self.timeline,
            intervals,
            ordinary_outcome: outcome,
            elapsed_ns,
            last_reap_ns: self.tree_reap_ns,
            first_error_ns: first.map_or(Metric::Unavailable(Unavailable::MissingEndpoint), Metric::Observed),
            error_drain_ns: drain,
            resources,
            unavailable_phases: vec![Unavailable::UnsupportedProtocol],
            reduction_error,
            sink_failed: false,
        };
        let publication_error = publish(&self.root.join("rust-waits.json"), &report)
            .err()
            .map(|_| "diagnostic_publication_failed".to_string());
        report.sink_failed = publication_error.is_some();
        DiagnosticOutcome {
            report,
            publication_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collector() -> (PathBuf, WaitCollector) {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("mantle-wait-origin-{}-{nonce}", std::process::id()));
        crate::process::private_dir(&root).unwrap();
        let context = WaitContext {
            invocation: Digest::from_slice(b"invocation"),
            clock: Digest::from_slice(b"clock"),
            toolchain: Digest::from_slice(b"toolchain"),
            configuration: Digest::from_slice(b"configuration"),
            host: Digest::from_slice(b"host"),
            target: Digest::from_slice(b"target"),
        };
        let collector = WaitCollector::start(&root, &std::env::current_exe().unwrap(), context).unwrap();
        (root, collector)
    }

    #[test]
    fn supervisor_elapsed_and_resource_offsets_use_the_wrapper_clock() {
        let (root, mut collector) = collector();
        let supervision_origin = collector.origin_ns + 1_000;
        let mut resources = ResourceReport::unavailable();
        resources.samples.push(ResourceSample {
            offset_ns: 7,
            rss_bytes: 4096,
            cpu_user_ns: 3,
            cpu_system_ns: 2,
            processes: 1,
        });
        resources.gaps.push(SampleGap {
            start_ns: 8,
            end_ns: Some(10),
            reason: Unavailable::Unsampled,
        });
        collector.set_resources(resources, supervision_origin);
        let result = collector.finish(RunOutcome::Success, supervision_origin, 20, Some(30));
        assert_eq!(result.report.elapsed_ns, 1_020);
        assert_eq!(result.report.last_reap_ns, Some(1_030));
        assert_eq!(result.report.resources.samples[0].offset_ns, 1_007);
        assert_eq!(result.report.resources.samples[0].cpu_user_ns, 3);
        assert_eq!(result.report.resources.gaps[0].start_ns, 0);
        assert_eq!(result.report.resources.gaps[0].end_ns, Some(1_000));
        assert_eq!(result.report.resources.gaps[1].start_ns, 1_008);
        assert_eq!(result.report.resources.gaps[1].end_ns, Some(1_010));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_precollector_supervision_origin_cannot_claim_complete_drain_timing() {
        let (root, collector) = collector();
        let invalid_origin = collector.origin_ns - 1;
        let result = collector.finish(RunOutcome::Rejected, invalid_origin, 20, Some(30));
        assert!(!result.report.timeline.complete);
        assert_eq!(result.report.last_reap_ns, None);
        assert!(matches!(result.report.error_drain_ns, Metric::Unavailable(_)));
        fs::remove_dir_all(root).unwrap();
    }
}
