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

pub use mantle_rust_differential_core::Digest;
pub use mantle_rust_differential_core::waits::*;
use serde::Deserialize;
use serde::Serialize;
const MAX_CAPTURE_FRAMES: usize = 4096;
const MAX_CAPTURE_BYTES: usize = 2_097_152;
const MAX_CORRELATIONS: usize = 256;
const RESERVED_TERMINAL_FRAMES: usize = MAX_CORRELATIONS.saturating_mul(2);
const RESERVED_TERMINAL_BYTES: usize = RESERVED_TERMINAL_FRAMES.saturating_mul(MAX_RECORD_BYTES);
const MAX_CARGO_FACTS: usize = 256;
const MAX_LINE_BYTES: usize = 64 * 1024;
const ENV_SOCKET: &str = "MANTLE_WAIT_SOCKET";
const ENV_CONTEXT: &str = "MANTLE_WAIT_CONTEXT";
const ENV_ORIGIN: &str = "MANTLE_WAIT_ORIGIN_NS";

/// Explicit shell selection, never part of a derivation, receipt, or cache identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeOptions {
    pub directory: PathBuf,
    pub wrapper: Option<PathBuf>,
    /// Late-bound admitted dynamic-plan facts; never serialized as runtime policy.
    #[serde(skip)]
    pub native_units: Option<Arc<std::sync::Mutex<BTreeMap<Digest, UnitIdentity>>>>,
}

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
        if !wrapper.is_absolute() || wrapper.components().any(|part| matches!(part, std::path::Component::ParentDir)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "wait wrapper must be an absolute declared path"));
        }
        let wrapper = wrapper.to_path_buf();
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
    /// Host socket for an execution-only sandbox mount, not a declared build input.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
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
    pub fn observation_lost(&mut self) {
        self.fact_overflow = self.fact_overflow.saturating_add(1);
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
        let has_bounded_samples = resources.samples.len() <= MAX_CAPTURE_FRAMES;
        let has_bounded_gaps = resources.gaps.len() <= MAX_CAPTURE_FRAMES;
        let has_valid_sampling =
            resources.interval_ns >= 100_000_000 && matches!(resources.scope.as_str(), "process_tree" | "cgroup");
        let has_valid_metric =
            matches!(resources.metric.as_str(), "sum_rss_bytes" | "memory_current_bytes" | "memory_peak_bytes");
        let has_bounded_storage = has_bounded_samples && has_bounded_gaps;
        let is_recognized_sampler = has_valid_sampling && has_valid_metric;
        if has_bounded_storage && is_recognized_sampler {
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
        remove_socket_best_effort(&self.socket_path);
        let mut diagnostic = assemble(&self.context, capture, &self.facts, Completion {
            overflow: self.fact_overflow,
            outcome,
            elapsed_ns,
            last_reap_ns,
        });
        diagnostic.resources = self.resources.clone();
        if diagnostic.resources.samples.is_empty() {
            diagnostic.resources.gaps.push(SampleGap {
                start_ns: 0,
                end_ns: Some(elapsed_ns),
                reason: Unavailable::Unsampled,
            });
        }
        let publication_error = publish(&self.root.join("rust-waits.json"), &diagnostic)
            .err()
            .map(|_| "diagnostic_publication_failed".to_string());
        diagnostic.sink_failed = publication_error.is_some();
        DiagnosticOutcome {
            report: diagnostic,
            publication_error,
        }
    }
}
impl Drop for WaitCollector {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.receiver.take() {
            handle.join().map(drop).unwrap_or_else(|_| {
                // Drop cannot recover diagnostic state after a receiver panic.
            });
        }
        remove_socket_best_effort(&self.socket_path);
    }
}
fn remove_socket_best_effort(path: &Path) {
    // Cleanup cannot change the ordinary result, including an already absent socket.
    fs::remove_file(path).unwrap_or(());
}
fn capture(socket: UnixDatagram, stop: Arc<AtomicBool>, context: WaitContext, origin_ns: u64) -> Capture {
    let mut result = Capture {
        records: Vec::new(),
        dropped: 0,
        malformed: 0,
        bytes: 0,
    };
    let mut buffer = [0u8; MAX_RECORD_BYTES.saturating_add(1)];
    let mut stopping_frames = MAX_CAPTURE_FRAMES;
    // Active lifetime belongs to the execution owner; only shutdown draining consumes this budget.
    while stopping_frames > 0 {
        if stop.load(Ordering::Acquire) {
            stopping_frames = stopping_frames.saturating_sub(1);
            if socket.set_nonblocking(true).is_err() {
                result.dropped = result.dropped.saturating_add(1);
                return result;
            }
        }
        match socket.recv(&mut buffer) {
            Ok(received_bytes) => {
                let receipt_ns = monotonic_ns().unwrap_or(origin_ns).saturating_sub(origin_ns);
                capture_datagram(&mut result, &buffer[..received_bytes], &context, receipt_ns);
            }
            // A timed receive is never restarted after a signal. EINTR, like a timeout, consumed no datagram.
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                ) =>
            {
                if stop.load(Ordering::Acquire) {
                    return result;
                }
            }
            Err(_) => {
                result.dropped = result.dropped.saturating_add(1);
                return result;
            }
        }
        assert!(result.records.len() <= MAX_CAPTURE_FRAMES);
        assert!(result.bytes <= MAX_CAPTURE_BYTES);
    }
    result.dropped = result.dropped.saturating_add(1);
    result
}
fn capture_datagram(result: &mut Capture, bytes: &[u8], context: &WaitContext, receipt_ns: u64) {
    if bytes.len() > MAX_RECORD_BYTES {
        result.dropped = result.dropped.saturating_add(1);
        return;
    }
    let frame = match serde_json::from_slice::<Frame>(bytes) {
        Ok(frame) if frame.invocation == context.invocation && frame.clock == context.clock => frame,
        _ => {
            result.malformed = result.malformed.saturating_add(1);
            return;
        }
    };
    if matches!(frame.kind, FrameKind::Dropped) {
        result.dropped = result.dropped.saturating_add(1);
        return;
    }
    if !valid_binding(frame.binding.as_ref()) {
        result.malformed = result.malformed.saturating_add(1);
        return;
    }
    let is_terminal = matches!(frame.kind, FrameKind::Exited { .. } | FrameKind::Reaped | FrameKind::SpawnFailed);
    let (max_frames, max_bytes) = capture_limits(is_terminal);
    if result.records.len() >= max_frames || result.bytes.saturating_add(bytes.len()) > max_bytes {
        result.dropped = result.dropped.saturating_add(1);
        return;
    }
    assert!(result.records.len() < max_frames);
    assert!(result.bytes.saturating_add(bytes.len()) <= max_bytes);
    result.bytes = result.bytes.saturating_add(bytes.len());
    result.records.push(Received { frame, receipt_ns });
}
fn capture_limits(is_terminal: bool) -> (usize, usize) {
    const {
        assert!(RESERVED_TERMINAL_FRAMES < MAX_CAPTURE_FRAMES);
        assert!(RESERVED_TERMINAL_BYTES < MAX_CAPTURE_BYTES);
    }
    if is_terminal {
        (MAX_CAPTURE_FRAMES, MAX_CAPTURE_BYTES)
    } else {
        (
            MAX_CAPTURE_FRAMES.saturating_sub(RESERVED_TERMINAL_FRAMES),
            MAX_CAPTURE_BYTES.saturating_sub(RESERVED_TERMINAL_BYTES),
        )
    }
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
    if manifest.len() > 4096 || source_path.len() > 4096 || package.len() > 4096 {
        return None;
    }
    if kind.len() > 8 || features.len() > MAX_PREREQUISITES {
        return None;
    }
    assert!(kind.len() <= 8);
    assert!(features.len() <= MAX_PREREQUISITES);
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
fn complete_attempts(timeline: &Timeline) -> bool {
    let mut states = BTreeMap::<&Digest, u8>::new();
    for event in &timeline.events {
        let bit = match &event.kind {
            EventKind::Started => 1,
            EventKind::Exited { .. } => 2,
            EventKind::Reaped => 4,
            EventKind::CacheRestored { .. } => 8,
            _ => 0,
        };
        *states.entry(&event.attempt).or_default() |= bit;
    }
    timeline
        .attempts
        .iter()
        .all(|attempt| states.get(&attempt.id).is_some_and(|bits| *bits == 7 || *bits == 8))
}

struct Completion {
    overflow: u64,
    outcome: RunOutcome,
    elapsed_ns: u64,
    last_reap_ns: Option<u64>,
}
fn assemble(context: &WaitContext, capture: Capture, facts: &[CargoFact], completion: Completion) -> WaitReport {
    assert!(capture.records.len() <= MAX_CAPTURE_FRAMES);
    assert!(facts.len() <= MAX_CARGO_FACTS);
    let mut timeline = Timeline {
        invocation: context.invocation.clone(),
        clock: context.clock.clone(),
        units: Vec::new(),
        attempts: Vec::new(),
        events: Vec::new(),
        dropped_records: capture.dropped.saturating_add(capture.malformed).saturating_add(completion.overflow),
        complete: capture.dropped == 0 && capture.malformed == 0 && completion.overflow == 0,
    };
    let (tokens, bindings) = collect_correlations(&mut timeline, &capture);
    let ids = assemble_attempts(&mut timeline, context, tokens, &bindings, facts);
    assemble_events(&mut timeline, &capture, &ids);
    for fact in facts.iter().filter(|fact| !fact.fresh) {
        if !bindings.values().any(|binding| matches_binding(binding, fact)) {
            timeline.complete = false;
            timeline.dropped_records = timeline.dropped_records.saturating_add(1);
        }
    }
    if timeline.attempts.is_empty() && completion.outcome != RunOutcome::Success {
        timeline.complete = false;
    }
    assembled_report(timeline, completion)
}
fn collect_correlations<'a>(
    timeline: &mut Timeline,
    capture: &'a Capture,
) -> (BTreeSet<&'a Digest>, BTreeMap<&'a Digest, &'a Binding>) {
    let mut bindings = BTreeMap::new();
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
    assert!(tokens.len() <= MAX_CORRELATIONS);
    assert!(bindings.len() <= tokens.len());
    (tokens, bindings)
}
fn assemble_attempts(
    timeline: &mut Timeline,
    context: &WaitContext,
    tokens: BTreeSet<&Digest>,
    bindings: &BTreeMap<&Digest, &Binding>,
    facts: &[CargoFact],
) -> BTreeMap<Digest, Digest> {
    let mut ids = Vec::with_capacity(tokens.len().min(MAX_CORRELATIONS));
    for token in tokens {
        if ids.len() >= MAX_CORRELATIONS {
            timeline.complete = false;
            timeline.dropped_records = timeline.dropped_records.saturating_add(1);
            break;
        }
        let binding = bindings.get(token).copied();
        let mut matches = facts.iter().filter(|fact| binding.is_some_and(|b| matches_binding(b, fact)));
        let first_match = matches.next();
        // Two launches fitting one artifact remain ambiguous; never use nearest time.
        let is_unique = matches.next().is_none()
            && first_match.is_some_and(|fact| bindings.values().filter(|b| matches_binding(b, fact)).count() == 1);
        let fact = first_match.filter(|_| is_unique);
        let role = attempt_role(binding, fact);
        let unit = assemble_unit(timeline, context, binding, fact, &role);
        let resolved_process = digest_json(
            "wait-resolved-wrapper-process-v1",
            &(token, binding.map(|b| (&b.arguments, &b.compiler)), &context.configuration),
        );
        let unmapped = if unit.is_some() {
            None
        } else if first_match.is_none() {
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
        assert!(ids.len() < MAX_CORRELATIONS);
        ids.push((token.clone(), attempt.id.clone()));
        timeline.attempts.push(attempt);
    }
    assert_eq!(ids.len(), timeline.attempts.len());
    // Tokens are a set, so every collected key is unique.
    ids.into_iter().collect()
}
/// A host-target fact, or a proc-macro compile of a source file, runs on the host.
/// Cargo's target-information probe reads its source from stdin while listing
/// every crate type, proc-macro included, so a launch without a source file
/// stays a tool probe whatever crate types it names.
fn attempt_role(binding: Option<&Binding>, fact: Option<&CargoFact>) -> ProcessRole {
    match (binding, fact) {
        (_, Some(f)) if f.host => ProcessRole::HostCompiler,
        (Some(b), _) if b.source_path.is_none() => ProcessRole::ToolProbe,
        (Some(b), _) if b.proc_macro => ProcessRole::HostCompiler,
        (Some(_), _) => ProcessRole::TargetCompiler,
        (None, _) => ProcessRole::ToolProbe,
    }
}
fn assemble_unit(
    timeline: &mut Timeline,
    context: &WaitContext,
    binding: Option<&Binding>,
    fact: Option<&CargoFact>,
    role: &ProcessRole,
) -> Option<Digest> {
    let (Some(binding), Some(fact)) = (binding, fact) else {
        return None;
    };
    assert!(binding.features.len() <= MAX_PREREQUISITES);
    assert!(timeline.units.len() < MAX_UNITS);
    let host = context.host.clone();
    let triple = if fact.host {
        host.clone()
    } else {
        context.target.clone()
    };
    let facts = UnitIdentity {
        package: fact.package.clone(),
        source: digest_json("wait-package-content-v1", &(&fact.source, &context.configuration)),
        target: fact.target.clone(),
        target_kind: fact.target_kind.clone(),
        host,
        target_triple: triple,
        role: role.clone(),
        features: binding.features.clone(),
        profile: fact.profile.clone(),
        mode: context.configuration.clone(),
        compiler: binding.compiler.clone(),
        arguments: binding.arguments.clone(),
        inputs: context.configuration.clone(),
        dependencies: Vec::new(),
    };
    let id = facts.identity().ok()?;
    timeline.units.push(Unit { id: id.clone(), facts });
    Some(id)
}
fn assemble_events(timeline: &mut Timeline, capture: &Capture, ids: &BTreeMap<Digest, Digest>) {
    let mut sequence = 0u64;
    let mut started = BTreeSet::new();
    let mut exited = BTreeSet::new();
    for received in &capture.records {
        let frame = &received.frame;
        let Some(id) = ids.get(&frame.token) else { continue };
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
            _ => {
                timeline.complete = false;
                continue;
            }
        };
        sequence = sequence.saturating_add(1);
        timeline.events.push(Event {
            reference: EventRef {
                invocation: timeline.invocation.clone(),
                clock: timeline.clock.clone(),
                sequence,
            },
            offset_ns: received.receipt_ns,
            attempt: id.clone(),
            source_sequence: Some(frame.sequence),
            kind,
        });
    }
    assert!(timeline.events.len() <= capture.records.len());
    assert!(exited.is_subset(&started));
    if started != exited || !complete_attempts(timeline) {
        timeline.complete = false;
    }
}
fn assembled_report(mut timeline: Timeline, completion: Completion) -> WaitReport {
    assert!(timeline.events.len() <= MAX_CAPTURE_FRAMES);
    assert!(timeline.attempts.len() <= MAX_CORRELATIONS);
    let first_error = timeline.events.iter().find(|e| matches!(e.kind, EventKind::ErrorObserved)).map(|e| e.offset_ns);
    let first_error_ns = first_error.map_or(Metric::Unavailable(Unavailable::MissingEndpoint), Metric::Observed);
    let error_drain_ns = drain_metric(DrainEndpoints {
        first_error_ns: first_error,
        last_reap_ns: completion.last_reap_ns,
    });
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
        ordinary_outcome: completion.outcome,
        elapsed_ns: completion.elapsed_ns,
        last_reap_ns: completion.last_reap_ns,
        first_error_ns,
        error_drain_ns,
        resources: ResourceReport::unavailable(),
        unavailable_phases: vec![Unavailable::UnsupportedProtocol],
        reduction_error,
        sink_failed: false,
    }
}
struct DrainEndpoints {
    first_error_ns: Option<u64>,
    last_reap_ns: Option<u64>,
}
fn drain_metric(endpoints: DrainEndpoints) -> Metric<u64> {
    match endpoints
        .first_error_ns
        .zip(endpoints.last_reap_ns)
        .and_then(|(start_ns, end_ns)| end_ns.checked_sub(start_ns))
    {
        Some(elapsed_ns) => Metric::Observed(elapsed_ns),
        None => Metric::Unavailable(Unavailable::Censored),
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
    let is_admitted = std::env::var_os("MANTLE_WAIT_ADMITTED_COMPILER")
        .is_none_or(|declared| Path::new(&compiler).is_absolute() && Path::new(&compiler) == Path::new(&declared));
    if !is_admitted {
        return wrapper_loss_then_exec(&compiler);
    }
    let args: Vec<OsString> = arguments.take(4097).collect();
    if args.len() > 4096 {
        return wrapper_loss_then_exec(&compiler);
    }
    assert!(is_admitted);
    assert!(args.len() <= 4096);
    let Some(mut observer) = WrapperObserver::from_environment(&compiler, &args, true) else {
        return exec_unobserved_compiler(&compiler);
    };
    if observer.binding.is_none() {
        observer.emit(FrameKind::Dropped, false);
        return exec_unobserved_compiler(&compiler);
    }
    let mut command = Command::new(&compiler);
    command.args(&args).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            observer.emit(FrameKind::SpawnFailed, false);
            return 127;
        }
    };
    observer.emit(FrameKind::Started, true);
    let stderr = child.stderr.take();
    let forwarding = thread::spawn(move || forward_compiler_stderr(stderr, observer));
    let status = child.wait();
    if let Ok(mut observer) = forwarding.join() {
        let outcome = match &status {
            Ok(status) if status.success() => RunOutcome::Success,
            Ok(status) if status.code().is_none() => RunOutcome::Crash,
            Ok(_) => RunOutcome::Rejected,
            Err(_) => RunOutcome::Infrastructure,
        };
        observer.emit(FrameKind::Exited { outcome }, false);
        if status.is_ok() {
            observer.emit(FrameKind::Reaped, false);
        }
    }
    wrapper_exit_status(status)
}

fn exec_unobserved_compiler(compiler: &OsStr) -> i32 {
    use std::os::unix::process::CommandExt as _;
    let _ = Command::new(compiler).args(std::env::args_os().skip(2)).exec();
    127
}

fn wrapper_loss_then_exec(compiler: &OsStr) -> i32 {
    if let Some(mut observer) = WrapperObserver::from_environment(compiler, &[], false) {
        observer.emit(FrameKind::Dropped, false);
    }
    exec_unobserved_compiler(compiler)
}

fn wrapper_exit_status(status: io::Result<std::process::ExitStatus>) -> i32 {
    use std::os::unix::process::ExitStatusExt as _;
    match status {
        Ok(status) => {
            if let Some(signal) = status.signal() {
                // SAFETY: replay the actual child signal, rather than changing it to a normal exit.
                unsafe {
                    libc::signal(signal, libc::SIG_DFL);
                    libc::raise(signal);
                }
                return 128 + signal;
            }
            status.code().unwrap_or(127)
        }
        Err(_) => 127,
    }
}

struct CompilerStderrWriter<'a> {
    output: io::StderrLock<'a>,
    observer: WrapperObserver,
    line: Vec<u8>,
    oversized: bool,
}

impl CompilerStderrWriter<'_> {
    fn finish_line(&mut self) {
        if !self.oversized && !self.line.is_empty() && is_error(&self.line) {
            self.observer.emit(FrameKind::Error, false);
        }
        self.line.clear();
        self.oversized = false;
    }
    fn observe_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            if *byte == b'\n' {
                self.finish_line();
            } else if !self.oversized {
                if self.line.len() < MAX_LINE_BYTES {
                    self.line.push(*byte);
                } else {
                    self.line.clear();
                    self.oversized = true;
                }
            }
        }
        debug_assert!(self.line.len() <= MAX_LINE_BYTES);
    }
}

impl Write for CompilerStderrWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // A downstream output error must not leave the ordinary compiler blocked on its pipe.
        self.output.write_all(bytes).unwrap_or(());
        self.observe_bytes(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        // Output is best effort; its failure cannot change the compiler outcome.
        self.output.flush().unwrap_or(());
        Ok(())
    }
}

fn forward_compiler_stderr(source: Option<std::process::ChildStderr>, observer: WrapperObserver) -> WrapperObserver {
    let output = io::stderr();
    let mut writer = CompilerStderrWriter {
        output: output.lock(),
        observer,
        line: Vec::new(),
        oversized: false,
    };
    if let Some(mut source) = source {
        // The execution owner decides process lifetime. Streaming has no diagnostic output/deadline cap.
        if io::copy(&mut source, &mut writer).is_err() {
            writer.observer.emit(FrameKind::Dropped, false);
        }
        writer.finish_line();
    } else {
        writer.observer.emit(FrameKind::Dropped, false);
    }
    writer.observer
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
    binding: Option<Binding>,
    dropped: bool,
}
impl WrapperObserver {
    fn from_environment(compiler: &OsStr, args: &[OsString], admitted: bool) -> Option<Self> {
        let context = std::env::var(ENV_CONTEXT).ok()?;
        if context.len() > MAX_RECORD_BYTES {
            return None;
        }
        let context: WaitContext = serde_json::from_str(&context).ok()?;
        let origin_ns = std::env::var(ENV_ORIGIN).ok()?.parse().ok()?;
        let socket = UnixDatagram::unbound().ok()?;
        socket.connect(std::env::var_os(ENV_SOCKET)?).ok()?;
        socket.set_nonblocking(true).ok()?;
        // Invocation-local process/clock identity is diagnostic, not authentication.
        // Ordinary sandboxes deliberately mask random devices.
        let launch_ns = monotonic_ns().ok()?;
        let token = digest_json("wait-wrapper-launch-v2", &(&context.invocation, std::process::id(), launch_ns));
        let binding = admitted.then(|| wrapper_binding(compiler, args, &context)).flatten();
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
            binding: if include_binding { self.binding.clone() } else { None },
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
struct WrapperArguments {
    source_path: Option<Digest>,
    features: Vec<Digest>,
    opt_level: String,
    debuginfo: String,
    is_test: bool,
    explicit_target: Option<Digest>,
    is_proc_macro: bool,
}
fn wrapper_binding(compiler: &OsStr, args: &[OsString], context: &WaitContext) -> Option<Binding> {
    let (parsed, arguments) = parse_wrapper_arguments(args)?;
    assert!(args.len() <= 4096);
    assert!(parsed.features.len() <= MAX_PREREQUISITES);
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .and_then(|dir| absolute_path(&PathBuf::from(dir).join("Cargo.toml")).ok())
        .map(|p| path_digest(&p));
    let compiler = compiler_digest(compiler, context)?;
    Some(Binding {
        manifest,
        source_path: parsed.source_path,
        features: parsed.features,
        opt_level: parsed.opt_level,
        debuginfo: parsed.debuginfo,
        test: parsed.is_test,
        explicit_target: parsed.explicit_target,
        proc_macro: parsed.is_proc_macro,
        arguments,
        compiler,
    })
}
fn parse_wrapper_arguments(args: &[OsString]) -> Option<(WrapperArguments, Digest)> {
    if args.len() > 4096 {
        return None;
    }
    let mut argument_hasher = blake3::Hasher::new();
    argument_hasher.update(b"mantle-rust-wait-raw-os-argv-v1\0");
    let mut parsed = WrapperArguments {
        source_path: None,
        features: Vec::new(),
        opt_level: "0".into(),
        debuginfo: "0".into(),
        is_test: false,
        explicit_target: None,
        is_proc_macro: false,
    };
    let mut argument_bytes = 0usize;
    for (index, arg) in args.iter().enumerate() {
        argument_bytes = argument_bytes.checked_add(arg.len())?;
        if argument_bytes > 1_048_576 {
            return None;
        }
        argument_hasher.update(&(arg.len() as u64).to_le_bytes());
        argument_hasher.update(arg.as_bytes());
        let next = args.get(index.saturating_add(1));
        observe_wrapper_flags(&mut parsed, arg, next)?;
        if !arg.as_bytes().starts_with(b"-") && Path::new(arg).extension() == Some(OsStr::new("rs")) {
            let path = absolute_path(Path::new(arg)).ok()?;
            if parsed.source_path.replace(path_digest(&path)).is_some() {
                return None;
            }
        }
    }
    assert!(argument_bytes <= 1_048_576);
    assert!(parsed.features.len() <= MAX_PREREQUISITES);
    parsed.features.sort();
    parsed.features.dedup();
    let arguments = Digest::from_slice(argument_hasher.finalize().as_bytes());
    Some((parsed, arguments))
}
fn observe_wrapper_flags(parsed: &mut WrapperArguments, arg: &OsStr, next: Option<&OsString>) -> Option<()> {
    assert!(parsed.features.len() <= MAX_PREREQUISITES);
    assert!(arg.len() <= 1_048_576);
    if arg.as_bytes() == b"--test" {
        parsed.is_test = true;
    }
    if arg.as_bytes() == b"--target" {
        parsed.explicit_target = next.map(|v| Digest::from_slice(v.as_bytes()));
    }
    if arg.as_bytes() == b"--crate-type" {
        parsed.is_proc_macro |= next.is_some_and(|v| v.as_bytes() == b"proc-macro");
    }
    let Some(text) = arg.to_str() else { return Some(()) };
    if text == "--crate-type=proc-macro" {
        parsed.is_proc_macro = true;
    }
    if let Some(value) = text.strip_prefix("--target=") {
        parsed.explicit_target = Some(Digest::from_slice(value.as_bytes()));
    }
    let next_text = next.and_then(|v| v.to_str());
    let codegen = if text == "-C" {
        next_text
    } else {
        text.strip_prefix("-C")
    };
    observe_wrapper_profile(parsed, codegen);
    if text == "--cfg"
        && let Some(value) = next_text.and_then(|v| v.strip_prefix("feature=\"")).and_then(|v| v.strip_suffix('"'))
    {
        if parsed.features.len() >= MAX_PREREQUISITES {
            return None;
        }
        parsed.features.push(Digest::from_slice(value.as_bytes()));
    }
    Some(())
}
fn observe_wrapper_profile(parsed: &mut WrapperArguments, codegen: Option<&str>) {
    if let Some(value) = codegen.and_then(|v| v.strip_prefix("opt-level=")) {
        parsed.opt_level = value.to_string();
    }
    if let Some(value) = codegen.and_then(|v| v.strip_prefix("debuginfo=")) {
        parsed.debuginfo = value.to_string();
    }
}
fn compiler_digest(compiler: &OsStr, context: &WaitContext) -> Option<Digest> {
    const MAX_COMPILER_BYTES: u64 = 536_870_912;
    let compiler_path = absolute_path(Path::new(compiler)).ok()?;
    let mut file = File::open(&compiler_path).ok()?;
    let mut compiler_hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536];
    let mut total_bytes = 0u64;
    while total_bytes <= MAX_COMPILER_BYTES {
        let read_bytes = file.read(&mut buffer).ok()?;
        if read_bytes == 0 {
            return Some(digest_json(
                "wait-compiler-binary-toolchain-v1",
                &(&context.toolchain, compiler_hasher.finalize().as_bytes()),
            ));
        }
        assert!(read_bytes <= buffer.len());
        total_bytes = total_bytes.checked_add(read_bytes as u64)?;
        if total_bytes > MAX_COMPILER_BYTES {
            return None;
        }
        assert!(total_bytes <= MAX_COMPILER_BYTES);
        compiler_hasher.update(&buffer[..read_bytes]);
    }
    None
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
#[derive(Clone, Copy)]
struct ProcessScale {
    ticks_per_second: u64,
    page_bytes: u64,
}
fn proc_tree(root_pid: u32) -> io::Result<Vec<Proc>> {
    // SAFETY: sysconf with constant selectors has no pointer preconditions.
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    let page_bytes = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if ticks_per_second <= 0 || page_bytes <= 0 {
        return Err(io::Error::other("resource clock unavailable"));
    }
    let ticks_per_second = u64::try_from(ticks_per_second).map_err(io::Error::other)?;
    let page_bytes = u64::try_from(page_bytes).map_err(io::Error::other)?;
    assert!(ticks_per_second > 0);
    assert!(page_bytes > 0);
    let all = read_process_table(ProcessScale {
        ticks_per_second,
        page_bytes,
    })?;
    select_process_tree(all, root_pid)
}
fn read_process_table(scale: ProcessScale) -> io::Result<Vec<Proc>> {
    const MAX_PROC_ENTRIES: usize = 32768;
    let mut all = Vec::new();
    let mut entry_count = 0u32;
    for entry in fs::read_dir("/proc")? {
        entry_count = entry_count.saturating_add(1);
        if entry_count > 32768 {
            return Err(io::Error::other("process table bound"));
        }
        let entry = entry?;
        let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let Some(process) = read_process_stat(&entry.path(), pid, scale)? else {
            continue;
        };
        if all.len() >= MAX_PROC_ENTRIES {
            return Err(io::Error::other("process table bound"));
        }
        all.push(process);
    }
    assert!(all.len() <= MAX_PROC_ENTRIES);
    assert!(u64::try_from(all.len()).is_ok_and(|count| count <= u64::from(entry_count)));
    Ok(all)
}
fn read_process_stat(path: &Path, pid: u32, scale: ProcessScale) -> io::Result<Option<Proc>> {
    let mut file = match File::open(path.join("stat")) {
        Ok(file) => file,
        Err(error) if matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied) => {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let mut text = String::new();
    Read::by_ref(&mut file).take(8193).read_to_string(&mut text)?;
    if text.len() > 8192 {
        return Err(io::Error::other("process record bound"));
    }
    Ok(parse_process_stat(&text, pid, scale))
}
fn parse_process_stat(text: &str, pid: u32, scale: ProcessScale) -> Option<Proc> {
    let end_index = text.rfind(')')?.checked_add(1)?;
    let fields: Vec<_> = text.get(end_index..)?.split_whitespace().collect();
    if fields.len() < 22 {
        return None;
    }
    assert!(fields.len() >= 22);
    assert!(end_index <= text.len());
    let parse = |index: usize| fields[index].parse::<u64>().ok();
    let parent_pid = u32::try_from(parse(1)?).ok()?;
    let user_ticks = parse(11)?;
    let system_ticks = parse(12)?;
    let start_ticks = parse(19)?;
    let rss_pages = parse(21)?;
    Some(Proc {
        pid,
        parent: parent_pid,
        start: start_ticks,
        rss: rss_pages.saturating_mul(scale.page_bytes),
        user: user_ticks.saturating_mul(1_000_000_000).checked_div(scale.ticks_per_second)?,
        system: system_ticks.saturating_mul(1_000_000_000).checked_div(scale.ticks_per_second)?,
    })
}
fn select_process_tree(all: Vec<Proc>, root_pid: u32) -> io::Result<Vec<Proc>> {
    let mut selected = BTreeSet::new();
    selected.insert(root_pid);
    for _ in 0..MAX_UNITS {
        let previous_process_count = selected.len();
        for process in &all {
            if selected.contains(&process.parent) {
                selected.insert(process.pid);
            }
        }
        if selected.len() > MAX_UNITS {
            return Err(io::Error::other("process tree bound"));
        }
        if previous_process_count == selected.len() {
            break;
        }
    }
    let result: Vec<_> = all.into_iter().filter(|p| selected.contains(&p.pid)).collect();
    if !result.iter().any(|p| p.pid == root_pid) {
        return Err(io::Error::other("process root absent"));
    }
    assert!(result.len() <= MAX_UNITS);
    assert!(result.iter().any(|p| p.pid == root_pid));
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
    tree_pending: bool,
    resources: Option<ResourceReport>,
    native_socket: UnixDatagram,
    native_socket_path: PathBuf,
    native_sequences: BTreeMap<Digest, (u64, u64)>,
}
impl NativeWaitCollector {
    pub fn start(root: &Path, invocation: Digest, clock: Digest) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "native wait root must be private"));
        }
        let native_socket_path = root.join("native.sock");
        let native_socket = UnixDatagram::bind(&native_socket_path)?;
        fs::set_permissions(&native_socket_path, fs::Permissions::from_mode(0o600))?;
        native_socket.set_nonblocking(true)?;
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
            tree_pending: false,
            resources: None,
            native_socket,
            native_socket_path,
            native_sequences: BTreeMap::new(),
        })
    }
    pub fn offset_ns(&self) -> u64 {
        monotonic_ns().unwrap_or(self.origin_ns).saturating_sub(self.origin_ns)
    }
    pub fn invocation(&self) -> &Digest {
        &self.timeline.invocation
    }
    pub fn clock(&self) -> &Digest {
        &self.timeline.clock
    }
    pub fn set_resources(&mut self, resources: ResourceReport) {
        self.resources = Some(resources);
    }
    pub fn socket_path(&self) -> &Path {
        &self.native_socket_path
    }
    pub fn native_context(&self, attempt: &Digest) -> Option<NativeContext> {
        self.timeline.attempts.iter().any(|known| &known.id == attempt).then(|| NativeContext {
            invocation: self.timeline.invocation.clone(),
            clock: self.timeline.clock.clone(),
            attempt: attempt.clone(),
            origin_ns: self.origin_ns,
        })
    }
    pub fn configure_helper(&self, command: &mut Command, attempt: &Digest) -> Result<(), WaitError> {
        let context = self.native_context(attempt).ok_or(WaitError::Dangling)?;
        let encoded = serde_json::to_string(&context).map_err(|_| WaitError::Malformed)?;
        command
            .env("MANTLE_NATIVE_WAIT_SOCKET", self.socket_path())
            .env("MANTLE_NATIVE_WAIT_CONTEXT", encoded);
        Ok(())
    }
    pub fn drain_native_frames(&mut self) {
        let mut buffer = [0u8; MAX_RECORD_BYTES.saturating_add(1)];
        for _ in 0..MAX_CAPTURE_FRAMES {
            match self.native_socket.recv(&mut buffer) {
                Ok(size) if size <= MAX_RECORD_BYTES => match serde_json::from_slice(&buffer[..size]) {
                    Ok(frame) => self.observe_native_frame(frame),
                    Err(_) => self.drop_record(),
                },
                Ok(_) => self.drop_record(),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.drop_record();
                    return;
                }
            }
        }
        self.drop_record();
    }
    pub fn observe_native_frame(&mut self, frame: NativeFrame) {
        let previous = self.native_sequences.get(&frame.attempt).copied();
        let expected = previous.map_or(Some(0), |(sequence, _)| sequence.checked_add(1));
        let is_admitted = self.timeline.attempts.iter().any(|attempt| attempt.id == frame.attempt);
        let has_wrong_context = frame.invocation != self.timeline.invocation || frame.clock != self.timeline.clock;
        if has_wrong_context || !is_admitted || Some(frame.sequence) != expected {
            self.drop_record();
            return;
        }
        let is_future_frame = frame.source_offset_ns > self.offset_ns();
        let is_reversed_frame = previous.is_some_and(|(_, offset)| offset > frame.source_offset_ns);
        if is_future_frame || is_reversed_frame {
            self.drop_record();
            return;
        }
        self.native_sequences.insert(frame.attempt.clone(), (frame.sequence, frame.source_offset_ns));
        let kind = match frame.event {
            NativeEvent::Started => EventKind::Started,
            NativeEvent::ErrorObserved => EventKind::ErrorObserved,
            NativeEvent::Exited { outcome } => EventKind::Exited { outcome },
            NativeEvent::Reaped => EventKind::Reaped,
        };
        self.record_event(&frame.attempt, kind, frame.source_offset_ns, Some(frame.sequence));
    }
    pub fn register_unit(&mut self, facts: UnitIdentity) -> Result<Digest, WaitError> {
        let id = facts.identity()?;
        if self.timeline.units.iter().any(|u| u.id == id) {
            return Ok(id);
        }
        if self.timeline.units.len() >= MAX_UNITS {
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
    /// Actual supervisor boundary. Helpers update tree lifetime without being
    /// mislabeled as compiler attempts or retrospective compiler phases.
    pub fn process_event(&mut self, attempt: Option<&Digest>, event: crate::observed_process::ProcessEvent) {
        self.drain_native_frames();
        match event {
            crate::observed_process::ProcessEvent::Spawned(_) => {
                self.tree_reap_ns = None;
                self.tree_pending = true;
                if let Some(id) = attempt {
                    self.started(id);
                }
            }
            crate::observed_process::ProcessEvent::Exited { code, signal, timeout } => {
                if let Some(id) = attempt {
                    self.exited(
                        id,
                        if timeout {
                            RunOutcome::Timeout
                        } else if signal.is_some() {
                            RunOutcome::Crash
                        } else if code == Some(0) {
                            RunOutcome::Success
                        } else {
                            RunOutcome::Rejected
                        },
                    );
                }
            }
            crate::observed_process::ProcessEvent::Reaped => {
                self.tree_pending = false;
                if let Some(id) = attempt {
                    self.reaped(id);
                }
                self.tree_reaped();
            }
        }
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
        self.record_event(attempt, kind, self.offset_ns(), None);
    }
    fn record_event(&mut self, attempt: &Digest, kind: EventKind, offset_ns: u64, source_sequence: Option<u64>) {
        let is_terminal = matches!(kind, EventKind::Exited { .. } | EventKind::Reaped);
        let (max_frames, max_bytes) = capture_limits(is_terminal);
        let sequence = (self.timeline.events.len() as u64).saturating_add(1);
        let event = Event {
            reference: EventRef {
                invocation: self.timeline.invocation.clone(),
                clock: self.timeline.clock.clone(),
                sequence,
            },
            offset_ns,
            attempt: attempt.clone(),
            source_sequence,
            kind,
        };
        let Ok(bytes) = serde_json::to_vec(&event) else {
            self.drop_record();
            return;
        };
        if self.timeline.events.len() >= max_frames
            || bytes.len() > MAX_RECORD_BYTES
            || self.bytes.saturating_add(bytes.len()) > max_bytes
        {
            self.drop_record();
            return;
        }
        self.bytes = self.bytes.saturating_add(bytes.len());
        self.timeline.events.push(event);
    }
    pub fn tree_reaped(&mut self) {
        self.tree_reap_ns = Some(self.offset_ns());
    }
    /// The adapter observed work whose exact diagnostic association was lost.
    pub fn observation_lost(&mut self) {
        self.drop_record();
    }
    fn drop_record(&mut self) {
        self.timeline.complete = false;
        self.timeline.dropped_records = self.timeline.dropped_records.saturating_add(1);
    }
    pub fn finish(mut self, outcome: RunOutcome) -> DiagnosticOutcome {
        self.drain_native_frames();
        remove_socket_best_effort(&self.native_socket_path);
        self.timeline.events.sort_by_key(|event| (event.offset_ns, event.reference.sequence));
        for (index, event) in self.timeline.events.iter_mut().enumerate() {
            event.reference.sequence = (index as u64).saturating_add(1);
        }
        let elapsed_ns = self.offset_ns();
        if self.tree_pending || !complete_attempts(&self.timeline) {
            self.timeline.complete = false;
        }
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
        let error_drain_ns = drain_metric(DrainEndpoints {
            first_error_ns: first,
            last_reap_ns: self.tree_reap_ns,
        });
        let resources = self.resources.unwrap_or_else(|| {
            let mut unavailable = ResourceReport::unavailable();
            unavailable.gaps.push(SampleGap {
                start_ns: 0,
                end_ns: Some(elapsed_ns),
                reason: Unavailable::Unsampled,
            });
            unavailable
        });
        let mut diagnostic = WaitReport {
            schema: SCHEMA.into(),
            timeline: self.timeline,
            intervals,
            ordinary_outcome: outcome,
            elapsed_ns,
            last_reap_ns: self.tree_reap_ns,
            first_error_ns: first.map_or(Metric::Unavailable(Unavailable::MissingEndpoint), Metric::Observed),
            error_drain_ns,
            resources,
            unavailable_phases: vec![Unavailable::UnsupportedProtocol],
            reduction_error,
            sink_failed: false,
        };
        let publication_error = publish(&self.root.join("rust-waits.json"), &diagnostic)
            .err()
            .map(|_| "diagnostic_publication_failed".to_string());
        diagnostic.sink_failed = publication_error.is_some();
        DiagnosticOutcome {
            report: diagnostic,
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
        crate::observed_process::private_dir(&root).unwrap();
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

    #[test]
    fn missing_reap_is_censored_and_optional_overflow_preserves_terminal_events() {
        for overflow in [false, true] {
            let (root, cargo_collector) = collector();
            let invocation = cargo_collector.context.invocation.clone();
            let clock = cargo_collector.context.clock.clone();
            drop(cargo_collector);
            let mut native = NativeWaitCollector::start(&root, invocation.clone(), clock.clone()).unwrap();
            let id = native
                .register_attempt(Attempt {
                    id: Digest::from_slice(b"unassigned"),
                    invocation,
                    clock,
                    unit: None,
                    role: ProcessRole::ToolProbe,
                    number: 1,
                    resolved_process: Digest::from_slice(b"test-observed-process"),
                    prior: None,
                    unmapped: Some(Unavailable::Unmapped),
                })
                .unwrap();
            native.started(&id);
            if overflow {
                for _ in 0..MAX_CAPTURE_FRAMES {
                    native.error_observed(&id);
                }
            }
            native.exited(&id, RunOutcome::Rejected);
            if overflow {
                native.reaped(&id);
            }
            let observed = native.finish(RunOutcome::Rejected);
            assert!(!observed.report.timeline.complete);
            assert_eq!(observed.report.ordinary_outcome, RunOutcome::Rejected);
            if overflow {
                assert!(observed.report.timeline.dropped_records > 0);
                assert!(observed.report.timeline.events.iter().any(|e| matches!(e.kind, EventKind::Reaped)));
            } else {
                assert_eq!(observed.report.last_reap_ns, None);
                assert!(matches!(observed.report.error_drain_ns, Metric::Unavailable(Unavailable::Censored)));
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn a_later_unreaped_helper_cannot_reuse_an_earlier_tree_reap() {
        let (root, cargo_collector) = collector();
        let invocation = cargo_collector.context.invocation.clone();
        let clock = cargo_collector.context.clock.clone();
        drop(cargo_collector);
        let mut native = NativeWaitCollector::start(&root, invocation, clock).unwrap();
        native.process_event(None, crate::observed_process::ProcessEvent::Spawned(1));
        native.process_event(None, crate::observed_process::ProcessEvent::Reaped);
        native.process_event(None, crate::observed_process::ProcessEvent::Spawned(2));
        let observed = native.finish(RunOutcome::Timeout);
        assert_eq!(observed.report.last_reap_ns, None);
        assert!(!observed.report.timeline.complete);
        assert_eq!(observed.report.ordinary_outcome, RunOutcome::Timeout);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_current_cargo_artifact_without_its_compiler_observation_is_incomplete() {
        let (root, mut collector) = collector();
        let identity = Digest::from_slice(b"current-artifact");
        collector.facts.push(CargoFact {
            manifest: identity.clone(),
            source_path: identity.clone(),
            package: identity.clone(),
            source: identity.clone(),
            target: identity.clone(),
            target_kind: identity.clone(),
            features: Vec::new(),
            profile: identity,
            opt_level: "0".into(),
            debuginfo: "0".into(),
            test: false,
            host: false,
            fresh: false,
        });
        let origin = collector.origin_ns;
        let report = collector.finish(RunOutcome::Success, origin, 0, None).report;
        assert!(!report.timeline.complete);
        assert!(report.timeline.attempts.is_empty(), "missing observations must not invent compiler attempts");
        assert_eq!(report.ordinary_outcome, RunOutcome::Success);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_helper_sequence_gap_cannot_claim_completed_compiler_lifetime() {
        let (root, cargo) = collector();
        let invocation = cargo.context.invocation.clone();
        let clock = cargo.context.clock.clone();
        drop(cargo);
        let mut native = NativeWaitCollector::start(&root, invocation.clone(), clock.clone()).unwrap();
        let attempt = native
            .register_attempt(Attempt {
                id: Digest::from_slice(b"unassigned"),
                invocation: invocation.clone(),
                clock: clock.clone(),
                unit: None,
                role: ProcessRole::ToolProbe,
                number: 1,
                resolved_process: Digest::from_slice(b"admitted-helper"),
                prior: None,
                unmapped: Some(Unavailable::Unmapped),
            })
            .unwrap();
        let sender = UnixDatagram::unbound().unwrap();
        sender.connect(native.socket_path()).unwrap();
        for (sequence, event) in [
            (0, NativeEvent::Started),
            (2, NativeEvent::Exited {
                outcome: RunOutcome::Success,
            }),
            (3, NativeEvent::Reaped),
        ] {
            let frame = NativeFrame {
                invocation: invocation.clone(),
                clock: clock.clone(),
                attempt: attempt.clone(),
                sequence,
                source_offset_ns: 0,
                event,
            };
            sender.send(&serde_json::to_vec(&frame).unwrap()).unwrap();
        }
        native.drain_native_frames();
        let report = native.finish(RunOutcome::Success).report;
        assert!(!report.timeline.complete);
        assert!(
            !report
                .timeline
                .events
                .iter()
                .any(|event| matches!(event.kind, EventKind::Exited { .. } | EventKind::Reaped))
        );
        assert!(matches!(report.error_drain_ns, Metric::Unavailable(Unavailable::Censored)));
        assert_eq!(report.ordinary_outcome, RunOutcome::Success);
        fs::remove_dir_all(root).unwrap();
    }

    fn binding_for(args: &[&str]) -> Binding {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let (parsed, arguments) = parse_wrapper_arguments(&args).unwrap();
        Binding {
            manifest: None,
            source_path: parsed.source_path,
            features: parsed.features,
            opt_level: parsed.opt_level,
            debuginfo: parsed.debuginfo,
            test: parsed.is_test,
            explicit_target: parsed.explicit_target,
            proc_macro: parsed.is_proc_macro,
            arguments,
            compiler: Digest::from_slice(b"rustc"),
        }
    }

    #[test]
    fn target_info_probe_listing_proc_macro_stays_a_tool_probe() {
        let probe = binding_for(&[
            "-",
            "--crate-name",
            "___",
            "--print=file-names",
            "--crate-type",
            "bin",
            "--crate-type",
            "rlib",
            "--crate-type",
            "dylib",
            "--crate-type",
            "cdylib",
            "--crate-type",
            "staticlib",
            "--crate-type",
            "proc-macro",
            "--print=sysroot",
            "--print=cfg",
        ]);
        assert!(probe.proc_macro);
        assert!(probe.source_path.is_none());
        assert_eq!(attempt_role(Some(&probe), None), ProcessRole::ToolProbe);
    }

    #[test]
    fn compiles_of_source_files_keep_their_host_and_target_roles() {
        let proc_macro = binding_for(&["--crate-name", "demo_macro", "src/lib.rs", "--crate-type", "proc-macro"]);
        assert!(proc_macro.source_path.is_some());
        assert_eq!(attempt_role(Some(&proc_macro), None), ProcessRole::HostCompiler);
        let library = binding_for(&["--crate-name", "demo", "src/lib.rs", "--crate-type", "lib"]);
        assert_eq!(attempt_role(Some(&library), None), ProcessRole::TargetCompiler);
        assert_eq!(attempt_role(None, None), ProcessRole::ToolProbe);
    }

    extern "C" fn ignore_interrupt(_signal: libc::c_int) {}

    #[test]
    fn an_interrupted_receive_keeps_capturing_compiler_frames() {
        use std::os::unix::thread::JoinHandleExt as _;
        let handler = ignore_interrupt as extern "C" fn(libc::c_int) as libc::sighandler_t;
        // SAFETY: the handler is empty, so it is async-signal-safe. SIGURG is ignored by default and
        // only this test sends it, to one collector receive thread.
        assert_ne!(unsafe { libc::signal(libc::SIGURG, handler) }, libc::SIG_ERR);
        let (root, collector) = collector();
        let sender = UnixDatagram::unbound().unwrap();
        sender.connect(collector.socket_path()).unwrap();
        let receiver = collector.receiver.as_ref().unwrap().as_pthread_t();
        // A receive with a timeout is never restarted: each interrupted wait fails with EINTR.
        for _ in 0..8 {
            // SAFETY: the receive thread is joinable and not yet joined, so its handle stays valid.
            assert_eq!(unsafe { libc::pthread_kill(receiver, libc::SIGURG) }, 0);
            thread::sleep(Duration::from_millis(5));
        }
        let binding = binding_for(&["--crate-name", "demo", "src/lib.rs", "--crate-type", "lib"]);
        let kinds = [
            FrameKind::Started,
            FrameKind::Exited {
                outcome: RunOutcome::Success,
            },
            FrameKind::Reaped,
        ];
        for (sequence, kind) in (1..).zip(kinds) {
            let frame = Frame {
                invocation: collector.context.invocation.clone(),
                clock: collector.context.clock.clone(),
                token: Digest::from_slice(b"interrupted-receive"),
                sequence,
                source_offset_ns: 0,
                binding: matches!(kind, FrameKind::Started).then(|| binding.clone()),
                kind,
            };
            sender.send(&serde_json::to_vec(&frame).unwrap()).unwrap();
        }
        let origin_ns = collector.origin_ns;
        let report = collector.finish(RunOutcome::Success, origin_ns, 10, Some(20)).report;
        assert_eq!(report.timeline.dropped_records, 0);
        assert!(report.timeline.complete);
        assert_eq!(report.timeline.events.len(), 3);
        fs::remove_dir_all(root).unwrap();
    }
}
