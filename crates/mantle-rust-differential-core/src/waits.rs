//! Pure, diagnostic-only invocation-local wait reduction. No timing is cache authority.
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Digest;

pub const SCHEMA: &str = "mantle-rust-wait-diagnostics-v1";
pub const MAX_UNITS: usize = 4096;
pub const MAX_ATTEMPTS: usize = 16384;
pub const MAX_RECORDS: usize = 65536;
pub const MAX_RECORD_BYTES: usize = 2048;
pub const MAX_SIDECAR_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_RUNS: usize = 1024;
pub const MAX_PREREQUISITES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    Success,
    Rejected,
    Crash,
    Cancelled,
    Timeout,
    Infrastructure,
    Unrun,
}

/// Execution-only binding supplied by an adapter that already admitted the attempt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeContext {
    pub invocation: Digest,
    pub clock: Digest,
    pub attempt: Digest,
    pub origin_ns: u64,
}

/// A helper can report only process facts, never readiness or scheduler authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeEvent {
    Started,
    ErrorObserved,
    Exited { outcome: RunOutcome },
    Reaped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeFrame {
    pub invocation: Digest,
    pub clock: Digest,
    pub attempt: Digest,
    pub sequence: u64,
    pub source_offset_ns: u64,
    pub event: NativeEvent,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessRole {
    TargetCompiler,
    HostCompiler,
    ToolProbe,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    HostExecutable,
    FullMetadata,
    Linkable,
    EarlyInterface,
    CompletionObserved,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unavailable {
    MissingEndpoint,
    MissingReadiness,
    MissingQueue,
    UnsupportedProtocol,
    Unmapped,
    AmbiguousAssociation,
    IncompleteCapture,
    Unsampled,
    Censored,
    TooFewSamples,
    PercentilesDisabled,
    MissingControls,
    MixedControls,
    EqualityUnqualified,
    WorkUnqualified,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "availability", content = "value", rename_all = "snake_case")]
pub enum Metric<T> {
    Observed(T),
    Unavailable(Unavailable),
}

/// All semantic identities are supplied by the shell; free-form labels are deliberately absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitIdentity {
    pub package: Digest,
    pub source: Digest,
    pub target: Digest,
    pub target_kind: Digest,
    pub host: Digest,
    pub target_triple: Digest,
    pub role: ProcessRole,
    pub features: Vec<Digest>,
    pub profile: Digest,
    pub mode: Digest,
    pub compiler: Digest,
    pub arguments: Digest,
    pub inputs: Digest,
    pub dependencies: Vec<Dependency>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub unit: Digest,
    pub role: ArtifactRole,
}
impl UnitIdentity {
    pub fn identity(&self) -> Result<Digest, WaitError> {
        if self.features.len() > MAX_PREREQUISITES || self.dependencies.len() > MAX_PREREQUISITES {
            return Err(WaitError::Bounds);
        }
        if self.features.windows(2).any(|w| w[0] >= w[1]) {
            return Err(WaitError::Identity);
        }
        let bytes = serde_json::to_vec(&("mantle-rust-wait-unit-v1", self)).map_err(|_| WaitError::Identity)?;
        Ok(Digest::from_slice(&bytes))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: Digest,
    pub facts: UnitIdentity,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub id: Digest,
    pub invocation: Digest,
    pub clock: Digest,
    pub unit: Option<Digest>,
    pub role: ProcessRole,
    pub number: u32,
    pub resolved_process: Digest,
    pub prior: Option<Digest>,
    pub unmapped: Option<Unavailable>,
}
impl Attempt {
    pub fn identity(&self) -> Digest {
        // Association can be established after receipt; it must not rewrite process identity.
        let bytes = serde_json::to_vec(&(
            "mantle-rust-wait-attempt-v1",
            &self.invocation,
            &self.clock,
            &self.role,
            self.number,
            &self.resolved_process,
            &self.prior,
        ))
        .unwrap_or_default();
        Digest::from_slice(&bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub producer: Digest,
    pub role: ArtifactRole,
    pub identity: Digest,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventRef {
    pub invocation: Digest,
    pub clock: Digest,
    pub sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventKind {
    Requested { prerequisites: Vec<Artifact> },
    Enqueued,
    Dispatched,
    Started,
    ArtifactReady { artifact: Artifact },
    Paused { pause: u32, awaiting: Vec<Artifact> },
    Resumed { pause: u32 },
    ErrorObserved,
    Exited { outcome: RunOutcome },
    Reaped,
    CacheRestored { receipt: Digest },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub reference: EventRef,
    pub offset_ns: u64,
    pub attempt: Digest,
    /// Producer-local order is independent of reception order and retained where present.
    pub source_sequence: Option<u64>,
    pub kind: EventKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    pub invocation: Digest,
    pub clock: Digest,
    pub units: Vec<Unit>,
    pub attempts: Vec<Attempt>,
    pub events: Vec<Event>,
    pub dropped_records: u64,
    pub complete: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntervalKind {
    ProcessLifetime,
    DependencyWait,
    ReadyQueue,
    PostReadinessGap,
    AwaitingArtifacts,
    ResumeDelay,
    ErrorDrain,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    pub attempt: Digest,
    pub kind: IntervalKind,
    pub start: Option<EventRef>,
    pub end: Option<EventRef>,
    pub duration_ns: Metric<u64>,
    pub prerequisites: Vec<Artifact>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitError {
    Bounds,
    Schema,
    Identity,
    CrossClock,
    Dangling,
    Ordering,
    Malformed,
    Role,
    Duplicate,
}

impl core::fmt::Display for WaitError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl core::error::Error for WaitError {}

fn interval(
    attempt: &Digest,
    kind: IntervalKind,
    start: Option<&Event>,
    end: Option<&Event>,
    prerequisites: Vec<Artifact>,
    reason: Unavailable,
) -> Result<Interval, WaitError> {
    let duration_ns = match (start, end) {
        (Some(a), Some(b)) => {
            if a.reference.invocation != b.reference.invocation || a.reference.clock != b.reference.clock {
                return Err(WaitError::CrossClock);
            }
            if a.offset_ns > b.offset_ns || a.reference.sequence > b.reference.sequence {
                return Err(WaitError::Ordering);
            }
            Metric::Observed(b.offset_ns - a.offset_ns)
        }
        _ => Metric::Unavailable(reason),
    };
    Ok(Interval {
        attempt: attempt.clone(),
        kind,
        start: start.map(|e| e.reference.clone()),
        end: end.map(|e| e.reference.clone()),
        duration_ns,
        prerequisites,
    })
}

/// Reject malformed references before attributing any duration. No endpoint substitution.
pub fn reduce(t: &Timeline) -> Result<Vec<Interval>, WaitError> {
    validate(t)?;
    let mut ready: BTreeMap<Artifact, &Event> = BTreeMap::new();
    for e in &t.events {
        if let EventKind::ArtifactReady { artifact } = &e.kind
            && ready.insert(artifact.clone(), e).is_some()
        {
            return Err(WaitError::Duplicate);
        }
    }
    let mut result = Vec::new();
    for a in &t.attempts {
        let events: Vec<&Event> = t.events.iter().filter(|e| e.attempt == a.id).collect();
        let find = |predicate: fn(&EventKind) -> bool| events.iter().copied().find(|e| predicate(&e.kind));
        let start = find(|k| matches!(k, EventKind::Started));
        let exit = find(|k| matches!(k, EventKind::Exited { .. }));
        let reap = find(|k| matches!(k, EventKind::Reaped));
        let error = find(|k| matches!(k, EventKind::ErrorObserved));
        let cached = find(|k| matches!(k, EventKind::CacheRestored { .. })).is_some();
        if !cached {
            result.push(interval(
                &a.id,
                IntervalKind::ProcessLifetime,
                start,
                exit,
                Vec::new(),
                Unavailable::Censored,
            )?);
        }
        if error.is_some() {
            result.push(interval(&a.id, IntervalKind::ErrorDrain, error, reap, Vec::new(), Unavailable::Censored)?);
        }
        for request in events.iter().filter(|e| matches!(e.kind, EventKind::Requested { .. })) {
            let EventKind::Requested { prerequisites } = &request.kind else {
                unreachable!()
            };
            let last = last_ready(prerequisites, &ready);
            // Already satisfied prerequisites entail no dependency wait, not a negative interval.
            let endpoint = last.map(|e| if e.offset_ns < request.offset_ns { *request } else { e });
            result.push(interval(
                &a.id,
                IntervalKind::DependencyWait,
                Some(request),
                endpoint,
                prerequisites.clone(),
                Unavailable::MissingReadiness,
            )?);
            let enqueue = find(|k| matches!(k, EventKind::Enqueued));
            let dispatch = find(|k| matches!(k, EventKind::Dispatched));
            result.push(interval(
                &a.id,
                IntervalKind::ReadyQueue,
                enqueue,
                dispatch,
                Vec::new(),
                Unavailable::MissingQueue,
            )?);
            result.push(interval(
                &a.id,
                IntervalKind::PostReadinessGap,
                endpoint,
                enqueue.or(dispatch).or(start),
                prerequisites.clone(),
                Unavailable::MissingEndpoint,
            )?);
        }
        for pause in events.iter().filter(|e| matches!(e.kind, EventKind::Paused { .. })) {
            let EventKind::Paused { pause: key, awaiting } = &pause.kind else {
                unreachable!()
            };
            let resume =
                events.iter().copied().find(|e| matches!(e.kind, EventKind::Resumed { pause } if pause == *key));
            let last = last_ready(awaiting, &ready);
            let endpoint = last.map(|e| if e.offset_ns < pause.offset_ns { *pause } else { e });
            result.push(interval(
                &a.id,
                IntervalKind::AwaitingArtifacts,
                Some(pause),
                endpoint,
                awaiting.clone(),
                Unavailable::MissingReadiness,
            )?);
            result.push(interval(
                &a.id,
                IntervalKind::ResumeDelay,
                endpoint,
                resume,
                awaiting.clone(),
                Unavailable::MissingReadiness,
            )?);
        }
    }
    Ok(result)
}
fn last_ready<'a>(refs: &[Artifact], ready: &BTreeMap<Artifact, &'a Event>) -> Option<&'a Event> {
    let mut last: Option<&Event> = None;
    for reference in refs {
        let e = *ready.get(reference)?;
        if last.is_none_or(|old| (e.offset_ns, e.reference.sequence) > (old.offset_ns, old.reference.sequence)) {
            last = Some(e);
        }
    }
    last
}

pub fn validate(t: &Timeline) -> Result<(), WaitError> {
    if t.units.len() > MAX_UNITS || t.attempts.len() > MAX_ATTEMPTS || t.events.len() > MAX_RECORDS {
        return Err(WaitError::Bounds);
    }
    if t.complete && t.dropped_records != 0 {
        return Err(WaitError::Malformed);
    }
    let mut units = BTreeMap::new();
    for u in &t.units {
        if u.id != u.facts.identity()? {
            return Err(WaitError::Identity);
        }
        if units.insert(&u.id, &u.facts).is_some() {
            return Err(WaitError::Duplicate);
        }
    }
    for u in &t.units {
        let mut seen = BTreeSet::new();
        for d in &u.facts.dependencies {
            if !units.contains_key(&d.unit) {
                return Err(WaitError::Dangling);
            }
            if d.unit == u.id || !seen.insert((&d.unit, &d.role)) {
                return Err(WaitError::Malformed);
            }
        }
    }
    let mut attempts = BTreeMap::new();
    for a in &t.attempts {
        if a.invocation != t.invocation || a.clock != t.clock {
            return Err(WaitError::CrossClock);
        }
        if a.number == 0 || a.id != a.identity() {
            return Err(WaitError::Identity);
        }
        if let Some(unit) = &a.unit {
            let facts = units.get(unit).ok_or(WaitError::Dangling)?;
            if a.unmapped.is_some() || facts.role != a.role {
                return Err(WaitError::Role);
            }
        } else if a.unmapped.is_none() {
            return Err(WaitError::Identity);
        }
        if attempts.insert(a.id.clone(), a).is_some() {
            return Err(WaitError::Duplicate);
        }
    }
    for a in &t.attempts {
        if let Some(prior) = &a.prior {
            let p = attempts.get(prior).ok_or(WaitError::Dangling)?;
            if p.unit != a.unit || p.role != a.role || p.number.checked_add(1) != Some(a.number) {
                return Err(WaitError::Identity);
            }
        } else if a.number != 1 {
            return Err(WaitError::Identity);
        }
    }
    let mut previous = (0, 0);
    let mut states: BTreeMap<&Digest, AttemptState> = BTreeMap::new();
    for e in &t.events {
        if e.reference.invocation != t.invocation || e.reference.clock != t.clock {
            return Err(WaitError::CrossClock);
        }
        if e.reference.sequence <= previous.0 || e.offset_ns < previous.1 {
            return Err(WaitError::Ordering);
        }
        previous = (e.reference.sequence, e.offset_ns);
        let a = attempts.get(&e.attempt).ok_or(WaitError::Dangling)?;
        let state = states.entry(&e.attempt).or_default();
        if let Some(seq) = e.source_sequence {
            if state.source_sequence.is_some_and(|old| seq <= old) {
                return Err(WaitError::Ordering);
            }
            state.source_sequence = Some(seq);
        }
        validate_transition(e, a, state, &attempts, &units)?;
    }
    Ok(())
}
#[derive(Default)]
struct AttemptState {
    requested: bool,
    queued: bool,
    dispatched: bool,
    started: bool,
    exited: bool,
    reaped: bool,
    cached: bool,
    pause: Option<u32>,
    pauses: BTreeSet<u32>,
    source_sequence: Option<u64>,
}
fn validate_transition(
    e: &Event,
    a: &Attempt,
    s: &mut AttemptState,
    attempts: &BTreeMap<Digest, &Attempt>,
    units: &BTreeMap<&Digest, &UnitIdentity>,
) -> Result<(), WaitError> {
    if s.reaped {
        return Err(WaitError::Malformed);
    }
    match &e.kind {
        EventKind::Requested { prerequisites } => {
            if s.requested || s.started || s.cached {
                return Err(WaitError::Malformed);
            }
            s.requested = true;
            validate_artifacts(prerequisites, a, attempts, units, true)?;
        }
        EventKind::Enqueued => {
            if !s.requested || s.queued || s.started {
                return Err(WaitError::Malformed);
            }
            s.queued = true;
        }
        EventKind::Dispatched => {
            if s.dispatched || s.started || s.cached {
                return Err(WaitError::Malformed);
            }
            s.dispatched = true;
        }
        EventKind::Started => {
            if s.started || s.exited || s.cached {
                return Err(WaitError::Malformed);
            }
            s.started = true;
        }
        EventKind::Exited { outcome } => {
            if !s.started || s.exited || *outcome == RunOutcome::Unrun {
                return Err(WaitError::Malformed);
            }
            s.exited = true;
        }
        EventKind::Reaped => {
            if !s.exited {
                return Err(WaitError::Malformed);
            }
            s.reaped = true;
        }
        EventKind::ErrorObserved => {
            if !s.started || s.exited {
                return Err(WaitError::Malformed);
            }
        }
        EventKind::CacheRestored { .. } => {
            if s.started || s.cached || s.exited {
                return Err(WaitError::Malformed);
            }
            s.cached = true;
        }
        EventKind::ArtifactReady { artifact } => {
            if artifact.producer != a.id || (!s.started && !s.cached) {
                return Err(WaitError::Role);
            }
            if s.exited && artifact.role != ArtifactRole::CompletionObserved {
                return Err(WaitError::Role);
            }
        }
        EventKind::Paused { pause, awaiting } => {
            if !s.started || s.exited || s.pause.is_some() || !s.pauses.insert(*pause) {
                return Err(WaitError::Malformed);
            }
            validate_artifacts(awaiting, a, attempts, units, true)?;
            s.pause = Some(*pause);
        }
        EventKind::Resumed { pause } => {
            if s.exited || s.pause != Some(*pause) {
                return Err(WaitError::Malformed);
            }
            s.pause = None;
        }
    }
    Ok(())
}
fn validate_artifacts(
    refs: &[Artifact],
    consumer: &Attempt,
    attempts: &BTreeMap<Digest, &Attempt>,
    units: &BTreeMap<&Digest, &UnitIdentity>,
    require_edges: bool,
) -> Result<(), WaitError> {
    if refs.is_empty() || refs.len() > MAX_PREREQUISITES {
        return Err(WaitError::Bounds);
    }
    let mut seen = BTreeSet::new();
    for artifact in refs {
        if !seen.insert(artifact) {
            return Err(WaitError::Duplicate);
        }
        let producer = attempts.get(&artifact.producer).ok_or(WaitError::Dangling)?;
        if producer.id == consumer.id {
            return Err(WaitError::Role);
        }
        if require_edges {
            let unit = consumer.unit.as_ref().and_then(|id| units.get(id)).ok_or(WaitError::Identity)?;
            let producer_unit = producer.unit.as_ref().ok_or(WaitError::Identity)?;
            if !unit.dependencies.iter().any(|d| &d.unit == producer_unit && d.role == artifact.role) {
                return Err(WaitError::Role);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSample {
    pub offset_ns: u64,
    pub rss_bytes: u64,
    pub cpu_user_ns: u64,
    pub cpu_system_ns: u64,
    pub processes: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleGap {
    pub start_ns: u64,
    pub end_ns: Option<u64>,
    pub reason: Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceReport {
    pub scope: String,
    pub metric: String,
    pub interval_ns: u64,
    pub samples: Vec<ResourceSample>,
    pub gaps: Vec<SampleGap>,
    pub peak_bytes: Metric<u64>,
    pub observed_lower_bound: bool,
    pub cpu_user_ns: Metric<u64>,
    pub cpu_system_ns: Metric<u64>,
    pub paused_resident: Metric<u32>,
    pub idle_resident_bytes: Metric<u64>,
    pub session_ready_ns: Metric<u64>,
    pub shutdown_ns: Metric<u64>,
    pub rss_shared_pages_may_double_count: bool,
}
impl ResourceReport {
    pub fn unavailable() -> Self {
        Self {
            scope: "process_tree".into(),
            metric: "sum_rss_bytes".into(),
            interval_ns: 100_000_000,
            samples: Vec::new(),
            gaps: Vec::new(),
            peak_bytes: Metric::Unavailable(Unavailable::Unsampled),
            observed_lower_bound: true,
            cpu_user_ns: Metric::Unavailable(Unavailable::Unsampled),
            cpu_system_ns: Metric::Unavailable(Unavailable::Unsampled),
            paused_resident: Metric::Unavailable(Unavailable::UnsupportedProtocol),
            idle_resident_bytes: Metric::Unavailable(Unavailable::Unsampled),
            session_ready_ns: Metric::Unavailable(Unavailable::MissingEndpoint),
            shutdown_ns: Metric::Unavailable(Unavailable::MissingEndpoint),
            rss_shared_pages_may_double_count: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaitReport {
    pub schema: String,
    pub timeline: Timeline,
    pub intervals: Vec<Interval>,
    pub ordinary_outcome: RunOutcome,
    pub elapsed_ns: u64,
    pub last_reap_ns: Option<u64>,
    pub first_error_ns: Metric<u64>,
    pub error_drain_ns: Metric<u64>,
    pub resources: ResourceReport,
    pub unavailable_phases: Vec<Unavailable>,
    pub reduction_error: Option<String>,
    pub sink_failed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Control {
    pub role: String,
    pub identity: Digest,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    Bootstrap,
    Setup,
    Warmup,
    Restore,
    Cold,
    FirstEdit,
    SubsequentEdit,
    Unchanged,
    CacheHit,
    CacheMiss,
    Invalidation,
    Error,
    Recovery,
    Teardown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheState {
    Clean,
    Warm,
    Incremental,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunPlan {
    pub id: Digest,
    pub pair: Option<u32>,
    pub treatment_b: bool,
    pub scenario: Scenario,
    pub controls: Vec<Control>,
    pub cache: CacheState,
    pub before: Digest,
    pub after: Digest,
    pub predecessor: Option<Digest>,
    pub session: Digest,
    pub expectation: Digest,
    pub clean_reference: Digest,
    pub required: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSample {
    pub plan: Digest,
    pub invocation: Option<Digest>,
    pub outcome: RunOutcome,
    pub elapsed_ns: Option<u64>,
    pub equality_qualified: bool,
    pub work_qualified: bool,
    pub exclusion: Option<Unavailable>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distribution {
    pub count: u32,
    pub median_ns: Metric<u64>,
    pub min_ns: Metric<u64>,
    pub max_ns: Metric<u64>,
    pub percentiles: Metric<Vec<u64>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub scenario: Scenario,
    pub a: Distribution,
    pub b: Distribution,
    pub paired_deltas_ns: Vec<i128>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcludedRun {
    pub plan: Digest,
    pub reason: Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerReport {
    pub plans: Vec<RunPlan>,
    pub raw_samples: Vec<RunSample>,
    pub summaries: Vec<ScenarioSummary>,
    pub exclusions: Vec<ExcludedRun>,
    pub required_complete: bool,
    pub comparison_qualified: bool,
    /// This implementation predeclares no percentiles, including for larger samples.
    pub distribution_policy: String,
}
const CONTROL_ROLES: [&str; 11] = [
    "compiler",
    "cargo",
    "flags",
    "source",
    "lock",
    "inputs",
    "configuration",
    "resources",
    "instrumentation",
    "cache",
    "provider",
];
fn controls(plan: &RunPlan) -> Result<BTreeMap<&str, &Digest>, WaitError> {
    if plan.controls.len() != CONTROL_ROLES.len() {
        return Err(WaitError::Identity);
    }
    let mut result = BTreeMap::new();
    for control in &plan.controls {
        if !CONTROL_ROLES.contains(&control.role.as_str())
            || result.insert(control.role.as_str(), &control.identity).is_some()
        {
            return Err(WaitError::Identity);
        }
    }
    Ok(result)
}
fn distribution(mut values: Vec<u64>) -> Distribution {
    values.sort_unstable();
    let count = values.len();
    let metric = |v: Option<u64>| v.map_or(Metric::Unavailable(Unavailable::TooFewSamples), Metric::Observed);
    let median = if count == 0 {
        None
    } else if count % 2 == 1 {
        Some(values[count / 2])
    } else {
        let a = values[count / 2 - 1];
        let b = values[count / 2];
        Some(a + (b - a) / 2)
    };
    Distribution {
        count: count as u32,
        median_ns: metric(median),
        min_ns: metric(values.first().copied()),
        max_ns: metric(values.last().copied()),
        percentiles: Metric::Unavailable(Unavailable::PercentilesDisabled),
    }
}
/// Five alternating AB/BA pairs per scenario/cache cohort; every planned slot survives.
/// Raw setup/failure/censored observations are never successful distribution samples.
pub fn reduce_ledger(plans: &[RunPlan], samples: &[RunSample]) -> Result<LedgerReport, WaitError> {
    if plans.len() > MAX_RUNS || samples.len() > MAX_RUNS || plans.is_empty() {
        return Err(WaitError::Bounds);
    }
    let mut by_id = BTreeMap::new();
    let mut groups: BTreeMap<Scenario, Vec<&RunPlan>> = BTreeMap::new();
    for p in plans {
        if by_id.insert(&p.id, p).is_some() {
            return Err(WaitError::Duplicate);
        }
        if p.pair.is_some_and(|n| n >= 5) {
            return Err(WaitError::Bounds);
        }
        if p.pair.is_some() {
            groups.entry(p.scenario.clone()).or_default().push(p);
        }
    }
    for p in plans {
        if let Some(prior) = &p.predecessor {
            let predecessor = by_id.get(prior).ok_or(WaitError::Dangling)?;
            if predecessor.session != p.session || predecessor.after != p.before {
                return Err(WaitError::Identity);
            }
        }
    }
    let mut observed = BTreeMap::new();
    let mut invocations = BTreeSet::new();
    for s in samples {
        if !by_id.contains_key(&s.plan) {
            return Err(WaitError::Dangling);
        }
        if observed.insert(&s.plan, s).is_some() {
            return Err(WaitError::Duplicate);
        }
        if s.outcome != RunOutcome::Unrun && (s.invocation.is_none() || s.elapsed_ns.is_none()) {
            return Err(WaitError::Malformed);
        }
        if let Some(invocation) = &s.invocation
            && !invocations.insert(invocation)
        {
            return Err(WaitError::Duplicate);
        }
    }
    let mut raw = Vec::with_capacity(plans.len());
    let mut exclusions = Vec::new();
    let mut required_complete = true;
    for p in plans {
        let s = observed.get(&p.id).copied().cloned().unwrap_or(RunSample {
            plan: p.id.clone(),
            invocation: None,
            outcome: RunOutcome::Unrun,
            elapsed_ns: None,
            equality_qualified: false,
            work_qualified: false,
            exclusion: Some(Unavailable::MissingEndpoint),
        });
        if p.required
            && (s.outcome != RunOutcome::Success || !s.equality_qualified || !s.work_qualified || s.exclusion.is_some())
        {
            required_complete = false;
        }
        if let Some(reason) = &s.exclusion {
            exclusions.push(ExcludedRun {
                plan: p.id.clone(),
                reason: reason.clone(),
            });
        }
        raw.push(s);
    }
    let mut summaries = Vec::new();
    for (scenario, group) in groups {
        if group.len() != 10 {
            return Err(WaitError::Malformed);
        }
        let mut av = Vec::new();
        let mut bv = Vec::new();
        let mut deltas = Vec::new();
        let reference = group[0];
        for pair in 0..5 {
            let rows: Vec<_> = group.iter().copied().filter(|p| p.pair == Some(pair)).collect();
            if rows.len() != 2 || rows[0].treatment_b != (pair % 2 == 1) || rows[1].treatment_b != (pair % 2 == 0) {
                return Err(WaitError::Ordering);
            }
            let mut pair_values = [None, None];
            for p in rows {
                let s = raw.iter().find(|s| s.plan == p.id).ok_or(WaitError::Dangling)?;
                let reason = match (controls(reference), controls(p)) {
                    (Ok(a), Ok(b))
                        if a == b
                            && p.cache == reference.cache
                            && p.before == reference.before
                            && p.after == reference.after
                            && p.expectation == reference.expectation
                            && p.clean_reference == reference.clean_reference =>
                    {
                        if !s.equality_qualified {
                            Some(Unavailable::EqualityUnqualified)
                        } else if !s.work_qualified {
                            Some(Unavailable::WorkUnqualified)
                        } else {
                            s.exclusion.clone()
                        }
                    }
                    (Err(_), _) | (_, Err(_)) => Some(Unavailable::MissingControls),
                    _ => Some(Unavailable::MixedControls),
                };
                if let Some(reason) = reason {
                    exclusions.push(ExcludedRun {
                        plan: p.id.clone(),
                        reason,
                    });
                    continue;
                }
                if s.outcome == RunOutcome::Success {
                    let value = s.elapsed_ns.ok_or(WaitError::Malformed)?;
                    if p.treatment_b {
                        bv.push(value);
                        pair_values[1] = Some(value);
                    } else {
                        av.push(value);
                        pair_values[0] = Some(value);
                    }
                }
            }
            if let [Some(a), Some(b)] = pair_values {
                deltas.push(i128::from(b) - i128::from(a));
            }
        }
        summaries.push(ScenarioSummary {
            scenario,
            a: distribution(av),
            b: distribution(bv),
            paired_deltas_ns: deltas,
        });
    }
    let comparison_qualified = required_complete && exclusions.is_empty();
    Ok(LedgerReport {
        plans: plans.to_vec(),
        raw_samples: raw,
        summaries,
        exclusions,
        required_complete,
        comparison_qualified,
        distribution_policy: "median_min_max_no_percentiles_v1".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::vec;

    use super::*;
    fn d(value: &str) -> Digest {
        Digest::from_slice(value.as_bytes())
    }
    fn unit(role: ProcessRole, dependencies: Vec<Dependency>) -> Unit {
        let facts = UnitIdentity {
            package: d("same-package"),
            source: d("content"),
            target: d("same-crate-name"),
            target_kind: d("lib"),
            host: d("host"),
            target_triple: d("target"),
            role,
            features: vec![],
            profile: d("debug"),
            mode: d("build"),
            compiler: d("rustc"),
            arguments: d("argv"),
            inputs: d("inputs"),
            dependencies,
        };
        Unit {
            id: facts.identity().unwrap(),
            facts,
        }
    }
    fn attempt(unit: &Unit, resolved: &str) -> Attempt {
        let mut a = Attempt {
            id: d("pending"),
            invocation: d("invocation"),
            clock: d("clock"),
            unit: Some(unit.id.clone()),
            role: unit.facts.role.clone(),
            number: 1,
            resolved_process: d(resolved),
            prior: None,
            unmapped: None,
        };
        a.id = a.identity();
        a
    }
    fn event(a: &Attempt, sequence: u64, offset_ns: u64, kind: EventKind) -> Event {
        Event {
            reference: EventRef {
                invocation: a.invocation.clone(),
                clock: a.clock.clone(),
                sequence,
            },
            offset_ns,
            attempt: a.id.clone(),
            source_sequence: None,
            kind,
        }
    }
    fn timeline() -> Timeline {
        let producer = unit(ProcessRole::HostCompiler, vec![]);
        let consumer = unit(ProcessRole::TargetCompiler, vec![
            Dependency {
                unit: producer.id.clone(),
                role: ArtifactRole::FullMetadata,
            },
            Dependency {
                unit: producer.id.clone(),
                role: ArtifactRole::Linkable,
            },
        ]);
        let p = attempt(&producer, "producer");
        let c = attempt(&consumer, "consumer");
        let metadata = Artifact {
            producer: p.id.clone(),
            role: ArtifactRole::FullMetadata,
            identity: d("metadata"),
        };
        let linkable = Artifact {
            producer: p.id.clone(),
            role: ArtifactRole::Linkable,
            identity: d("rlib"),
        };
        let events = vec![
            event(&p, 1, 0, EventKind::Started),
            event(&c, 2, 10, EventKind::Requested {
                prerequisites: vec![metadata.clone()],
            }),
            event(&p, 3, 40, EventKind::ArtifactReady {
                artifact: metadata.clone(),
            }),
            event(&c, 4, 45, EventKind::Enqueued),
            event(&c, 5, 70, EventKind::Dispatched),
            event(&c, 6, 70, EventKind::Started),
            event(&c, 7, 100, EventKind::Paused {
                pause: 1,
                awaiting: vec![metadata, linkable.clone()],
            }),
            event(&p, 8, 150, EventKind::ArtifactReady { artifact: linkable }),
            event(&c, 9, 180, EventKind::Resumed { pause: 1 }),
            event(&p, 10, 200, EventKind::Exited {
                outcome: RunOutcome::Success,
            }),
            event(&c, 11, 220, EventKind::Exited {
                outcome: RunOutcome::Success,
            }),
        ];
        Timeline {
            invocation: d("invocation"),
            clock: d("clock"),
            units: vec![producer, consumer],
            attempts: vec![p, c],
            events,
            dropped_records: 0,
            complete: true,
        }
    }
    fn duration(rows: &[Interval], kind: IntervalKind) -> Metric<u64> {
        rows.iter().find(|r| r.kind == kind).unwrap().duration_ns.clone()
    }
    #[test]
    fn separates_dependency_queue_gap_and_exact_role_pause() {
        let t = timeline();
        let result = reduce(&t).unwrap();
        assert_eq!(duration(&result, IntervalKind::DependencyWait), Metric::Observed(30));
        assert_eq!(duration(&result, IntervalKind::ReadyQueue), Metric::Observed(25));
        assert_eq!(duration(&result, IntervalKind::PostReadinessGap), Metric::Observed(5));
        assert_eq!(duration(&result, IntervalKind::AwaitingArtifacts), Metric::Observed(50));
        assert_eq!(duration(&result, IntervalKind::ResumeDelay), Metric::Observed(30));
        let mut wrong_role = t;
        if let EventKind::Paused { awaiting, .. } = &mut wrong_role.events[6].kind {
            awaiting[1].role = ArtifactRole::EarlyInterface;
        }
        assert_eq!(reduce(&wrong_role), Err(WaitError::Role));
    }
    #[test]
    fn missing_readiness_queue_exit_remain_unavailable() {
        let mut t = timeline();
        t.events.retain(|e| {
            !matches!(
                e.kind,
                EventKind::Enqueued
                    | EventKind::ArtifactReady {
                        artifact: Artifact {
                            role: ArtifactRole::Linkable,
                            ..
                        }
                    }
                    | EventKind::Exited { .. }
            )
        });
        t.complete = false;
        let result = reduce(&t).unwrap();
        assert_eq!(duration(&result, IntervalKind::ReadyQueue), Metric::Unavailable(Unavailable::MissingQueue));
        assert_eq!(duration(&result, IntervalKind::ResumeDelay), Metric::Unavailable(Unavailable::MissingReadiness));
        assert_eq!(duration(&result, IntervalKind::ProcessLifetime), Metric::Unavailable(Unavailable::Censored));
    }
    #[test]
    fn rejects_cross_clock_dangling_and_malformed_events() {
        let mut t = timeline();
        t.events[1].reference.clock = d("another-host");
        assert_eq!(reduce(&t), Err(WaitError::CrossClock));
        t = timeline();
        t.events[1].attempt = d("missing");
        assert_eq!(reduce(&t), Err(WaitError::Dangling));
        t = timeline();
        t.events[6].kind = EventKind::Resumed { pause: 1 };
        assert_eq!(reduce(&t), Err(WaitError::Malformed));
        t = timeline();
        t.events[3].offset_ns = 39;
        assert_eq!(reduce(&t), Err(WaitError::Ordering));
    }
    #[test]
    fn separates_same_name_roles_retries_and_cache_without_compiler() {
        let host = unit(ProcessRole::HostCompiler, vec![]);
        let target = unit(ProcessRole::TargetCompiler, vec![]);
        assert_ne!(host.id, target.id);
        let original = attempt(&target, "failed");
        let mut retry = attempt(&target, "fallback");
        retry.number = 2;
        retry.prior = Some(original.id.clone());
        retry.id = retry.identity();
        assert_ne!(original.id, retry.id);
        let t = Timeline {
            invocation: original.invocation.clone(),
            clock: original.clock.clone(),
            units: vec![target],
            attempts: vec![original.clone(), retry.clone()],
            events: vec![
                event(&original, 1, 10, EventKind::Started),
                event(&original, 2, 20, EventKind::Exited {
                    outcome: RunOutcome::Rejected,
                }),
                event(&retry, 3, 25, EventKind::CacheRestored {
                    receipt: d("complete-receipt"),
                }),
            ],
            dropped_records: 0,
            complete: true,
        };
        let rows = reduce(&t).unwrap();
        assert_eq!(rows.iter().filter(|r| r.kind == IntervalKind::ProcessLifetime).count(), 1);
        assert_eq!(rows[0].duration_ns, Metric::Observed(10));
    }
    fn ledger() -> (Vec<RunPlan>, Vec<RunSample>) {
        let mut plans = Vec::new();
        let mut samples = Vec::new();
        for pair in 0..5 {
            for ordinal in 0_u32..2 {
                let id = d(&format!("plan-{pair}-{ordinal}"));
                plans.push(RunPlan {
                    id: id.clone(),
                    pair: Some(pair),
                    treatment_b: ordinal == if pair % 2 == 0 { 1 } else { 0 },
                    scenario: Scenario::FirstEdit,
                    controls: CONTROL_ROLES
                        .iter()
                        .map(|role| Control {
                            role: (*role).into(),
                            identity: d(role),
                        })
                        .collect(),
                    cache: CacheState::Incremental,
                    before: d("before"),
                    after: d("after"),
                    predecessor: None,
                    session: d("session"),
                    expectation: d("oracle"),
                    clean_reference: d("clean"),
                    required: true,
                });
                samples.push(RunSample {
                    plan: id,
                    invocation: Some(d(&format!("invocation-{pair}-{ordinal}"))),
                    outcome: RunOutcome::Success,
                    elapsed_ns: Some(100 + u64::from(pair) * 10 + u64::from(ordinal)),
                    equality_qualified: true,
                    work_qualified: true,
                    exclusion: None,
                });
            }
        }
        (plans, samples)
    }
    #[test]
    fn retains_mixed_controls_failures_unrun_and_no_sparse_percentiles() {
        let (mut plans, mut samples) = ledger();
        plans[2].controls[0].identity = d("different-compiler");
        samples[7].outcome = RunOutcome::Timeout;
        let report = reduce_ledger(&plans, &samples).unwrap();
        assert_eq!(report.raw_samples[7].outcome, RunOutcome::Timeout);
        assert!(!report.comparison_qualified);
        assert!(!report.required_complete);
        assert!(report.exclusions.iter().any(|r| r.reason == Unavailable::MixedControls));
        let summary = &report.summaries[0];
        assert_eq!(summary.a.count + summary.b.count, 8);
        assert_eq!(summary.paired_deltas_ns.len(), 3);
        assert_eq!(summary.a.percentiles, Metric::Unavailable(Unavailable::PercentilesDisabled));
        samples.pop();
        let report = reduce_ledger(&plans, &samples).unwrap();
        assert_eq!(report.raw_samples[9].outcome, RunOutcome::Unrun);
        plans[0].controls.push(Control {
            role: "diagnostic_timing".into(),
            identity: d("nondeterministic"),
        });
        assert!(
            reduce_ledger(&plans, &samples)
                .unwrap()
                .exclusions
                .iter()
                .any(|r| r.reason == Unavailable::MissingControls)
        );
    }
}
