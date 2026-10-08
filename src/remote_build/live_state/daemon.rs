//! Transient, process-local live build facts over a Unix socket.
//!
//! Publisher connections own only the facts they opened; a lost publisher retracts
//! its facts. No fact is persisted or promoted to coordinator authority.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::net::Shutdown;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TrySendError;
use std::sync::mpsc::{self};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

use super::LIVE_BUILD_FACT_SCHEMA;
use super::LiveBuildFact;
use super::LiveBuildFactValue;
use super::LiveGoalPhase;
use super::LiveTerminalOutcome;
use super::MAX_LIVE_FACTS;
use super::admit_fact_size;
use super::admit_identity;
use super::fact_identity;

pub const LIVE_PROTOCOL_SCHEMA: &str = "mantle-live-build-protocol-v1";
const READINESS_SCHEMA: &str = crunch_service_readiness_core::READINESS_SCHEMA;
const MAX_FRAME_BYTES: usize = 4_096;
const MAX_SUBSCRIBERS: usize = 32;
const MAX_CONNECTIONS: usize = 128;
const MAX_FILTER_ITEMS: usize = 64;
const SUBSCRIBER_QUEUE: usize = 64;
const IO_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
enum Request {
    Open {
        schema: String,
        owner_run_id: String,
    },
    Publish {
        schema: String,
        fact_id: String,
        fact: IncomingFact,
    },
    Retract {
        schema: String,
        fact_id: String,
    },
    Stop {
        schema: String,
    },
    Subscribe {
        schema: String,
        filter: LiveFilter,
    },
}

impl Request {
    fn schema(&self) -> &str {
        match self {
            Self::Open { schema, .. }
            | Self::Publish { schema, .. }
            | Self::Retract { schema, .. }
            | Self::Stop { schema }
            | Self::Subscribe { schema, .. } => schema,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReadinessStageRequirements {
    require_action_reconciliation: bool,
    require_v2_receipt: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReadinessComponent {
    id: String,
    kind: String,
    depends_on: Vec<String>,
    user_states: Vec<String>,
    restart_policy: Option<String>,
    stage_requirements: Option<ReadinessStageRequirements>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReadinessProofCompletion {
    stage_evidence_digest_blake3: String,
    output_digest_blake3: String,
    execution_verified: bool,
    action_reconciled: bool,
    v2_receipt_verified: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReadinessObservation {
    id: String,
    generation: u32,
    states: Vec<String>,
    exit: Option<String>,
    request_acknowledged: bool,
    proof_completion: Option<ReadinessProofCompletion>,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
enum ReadinessRequest {
    Publish {
        schema: String,
        component: ReadinessComponent,
        observation: ReadinessObservation,
    },
    Query {
        schema: String,
        components: Vec<ReadinessComponent>,
    },
}

impl ReadinessRequest {
    fn schema(&self) -> &str {
        match self {
            Self::Publish { schema, .. } | Self::Query { schema, .. } => schema,
        }
    }
}

#[derive(Serialize)]
struct ReadinessPublishFrame<'a> {
    schema: &'static str,
    op: &'static str,
    component: &'a ReadinessComponent,
    observation: &'a ReadinessObservation,
}

#[derive(Serialize)]
struct ReadinessQueryFrame {
    schema: &'static str,
    op: &'static str,
    components: [ReadinessComponent; 0],
}

struct ReadinessRecord {
    component: ReadinessComponent,
    observation: Option<ReadinessObservation>,
    owner: Option<u64>,
    sequence: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IncomingFact {
    schema: String,
    value: LiveBuildFactValue,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveFilter {
    pub owner_run_ids: Vec<String>,
    pub job_ids: Vec<String>,
    pub kinds: Vec<String>,
}

struct Filter {
    owners: BTreeSet<String>,
    jobs: BTreeSet<String>,
    kinds: BTreeSet<String>,
}

impl Filter {
    fn valid(input: &LiveFilter) -> bool {
        input.owner_run_ids.len() + input.job_ids.len() + input.kinds.len() <= MAX_FILTER_ITEMS
            && input.owner_run_ids.iter().chain(&input.job_ids).all(|id| admit_identity(id).is_ok())
            && input
                .kinds
                .iter()
                .all(|kind| matches!(kind.as_str(), "worker-presence" | "goal" | "reservation" | "terminal-outcome"))
    }

    fn parse(input: LiveFilter) -> Option<Self> {
        if !Self::valid(&input) {
            return None;
        }
        Some(Self {
            owners: input.owner_run_ids.into_iter().collect(),
            jobs: input.job_ids.into_iter().collect(),
            kinds: input.kinds.into_iter().collect(),
        })
    }

    fn matches(&self, owner: &str, value: &LiveBuildFactValue) -> bool {
        if !self.owners.is_empty() && !self.owners.contains(owner) {
            return false;
        }
        let (kind, job) = match value {
            LiveBuildFactValue::WorkerPresence { .. } => ("worker-presence", None),
            LiveBuildFactValue::Goal { job_id, .. } => ("goal", Some(job_id.as_str())),
            LiveBuildFactValue::Reservation { job_id, .. } => ("reservation", Some(job_id.as_str())),
            LiveBuildFactValue::TerminalOutcome { job_id, .. } => ("terminal-outcome", Some(job_id.as_str())),
        };
        (self.kinds.is_empty() || self.kinds.contains(kind))
            && (self.jobs.is_empty() || job.is_some_and(|job| self.jobs.contains(job)))
    }
}

fn valid_fact_value(owner_run_id: &str, value: &LiveBuildFactValue) -> bool {
    let valid = |id: &str| admit_identity(id).is_ok();
    let local_endpoint = |endpoint: &str| {
        endpoint
            .strip_prefix("local-worker:")
            .is_some_and(|digest| digest == blake3::hash(owner_run_id.as_bytes()).to_hex().as_str())
    };
    match value {
        LiveBuildFactValue::WorkerPresence {
            endpoint_id,
            generation,
        } => {
            valid(endpoint_id)
                && *generation > 0
                && (!endpoint_id.starts_with("local-worker:") || (*generation == 1 && local_endpoint(endpoint_id)))
        }
        LiveBuildFactValue::Goal {
            job_id,
            phase,
            worker_endpoint_id,
            attempt_id,
            fence_generation,
        } => {
            valid(job_id)
                && worker_endpoint_id.as_deref().is_none_or(valid)
                && attempt_id.as_deref().is_none_or(valid)
                && fence_generation.is_none_or(|generation| generation > 0)
                && match phase {
                    LiveGoalPhase::Discovered => {
                        worker_endpoint_id.is_none() && attempt_id.is_some() == fence_generation.is_some()
                    }
                    LiveGoalPhase::Dispatched => {
                        // A local Worker has no remote attempt or fence; its
                        // endpoint must match this exact owner invocation.
                        worker_endpoint_id.is_some()
                            && ((attempt_id.is_some() && fence_generation.is_some())
                                || (attempt_id.is_none()
                                    && fence_generation.is_none()
                                    && worker_endpoint_id.as_deref().is_some_and(local_endpoint)))
                    }
                }
        }
        LiveBuildFactValue::Reservation {
            lease_id_blake3,
            job_id,
            worker_endpoint_id,
            attempt_id,
            fence_generation,
        } => {
            lease_id_blake3.len() == 64
                && lease_id_blake3.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                && valid(job_id)
                && valid(worker_endpoint_id)
                && valid(attempt_id)
                && *fence_generation > 0
        }
        LiveBuildFactValue::TerminalOutcome {
            job_id,
            attempt_id,
            outcome,
            terminal_phase,
        } => {
            valid(job_id)
                && attempt_id.as_deref().is_none_or(valid)
                && match terminal_phase.as_deref() {
                    None => matches!(outcome, LiveTerminalOutcome::Finished | LiveTerminalOutcome::Lost),
                    Some(phase) => matches!(phase, "evaluation" | "conversion" | "build" | "coordination"),
                }
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
enum Event {
    SnapshotStart {
        schema: &'static str,
    },
    SnapshotEnd {
        schema: &'static str,
    },
    Publish {
        schema: &'static str,
        owner_run_id: String,
        fact_id: String,
        fact: LiveBuildFact,
    },
    Retract {
        schema: &'static str,
        owner_run_id: String,
        fact_id: String,
    },
}

struct Entry {
    owner: String,
    fact: LiveBuildFact,
}

struct Subscriber {
    filter: Filter,
    sender: SyncSender<Event>,
    alive: Arc<AtomicBool>,
}

#[derive(Default)]
struct State {
    next_client: u64,
    owners: BTreeMap<String, u64>,
    facts: BTreeMap<String, Entry>,
    subscribers: BTreeMap<u64, Subscriber>,
    readiness_records: BTreeMap<String, ReadinessRecord>,
    next_readiness_sequence: u64,
    connections: usize,
    shutting_down: bool,
}

impl State {
    fn broadcast(&mut self, event: Event, owner: &str, value: &LiveBuildFactValue) {
        self.subscribers.retain(|_, subscriber| {
            if !subscriber.filter.matches(owner, value) {
                return true;
            }
            match subscriber.sender.try_send(event.clone()) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                    subscriber.alive.store(false, Ordering::Release);
                    false
                }
            }
        });
    }

    fn retract(&mut self, owner: &str, id: &str) {
        if let Some(entry) = self.facts.remove(id) {
            self.broadcast(
                Event::Retract {
                    schema: LIVE_PROTOCOL_SCHEMA,
                    owner_run_id: owner.to_owned(),
                    fact_id: id.to_owned(),
                },
                owner,
                &entry.fact.value,
            );
        }
    }

    fn close_owner(&mut self, owner: &str, client: u64) {
        if self.owners.get(owner) != Some(&client) {
            return;
        }
        self.owners.remove(owner);
        let ids: Vec<_> =
            self.facts.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, _)| id.clone()).collect();
        for id in ids {
            self.retract(owner, &id);
        }
    }
}

struct SocketCleanup {
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        if let Ok(metadata) = fs::symlink_metadata(&self.path)
            && metadata.dev() == self.device
            && metadata.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn bind_socket(path: &Path) -> io::Result<UnixListener> {
    match UnixListener::bind(path) {
        Ok(listener) => Ok(listener),
        Err(occupied) if occupied.kind() == io::ErrorKind::AddrInUse => {
            let old = fs::symlink_metadata(path)?;
            // Never remove a non-socket, another user's socket, or a live daemon.
            if !old.file_type().is_socket() || old.uid() != unsafe { libc::geteuid() } {
                return Err(occupied);
            }
            match UnixStream::connect(path) {
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {}
                _ => return Err(occupied),
            }
            let current = fs::symlink_metadata(path)?;
            if old.dev() != current.dev() || old.ino() != current.ino() {
                return Err(occupied);
            }
            fs::remove_file(path)?;
            UnixListener::bind(path)
        }
        Err(error) => Err(error),
    }
}

fn lock_state(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Serialize)]
struct SubscriptionRequest<'a> {
    schema: &'static str,
    op: &'static str,
    filter: &'a LiveFilter,
}

/// Follow a daemon subscription, copying newline-delimited events to `output`
/// until the daemon disconnects. An empty filter field matches all values.
pub fn subscribe(socket: &Path, filter: LiveFilter, output: &mut impl Write) -> io::Result<()> {
    if !Filter::valid(&filter) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid live-build filter"));
    }
    let mut stream = UnixStream::connect(socket)?;
    let request = SubscriptionRequest {
        schema: LIVE_PROTOCOL_SCHEMA,
        op: "subscribe",
        filter: &filter,
    };
    serde_json::to_writer(&mut stream, &request).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    let mut chunk = [0_u8; MAX_FRAME_BYTES];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(count) => {
                output.write_all(&chunk[..count])?;
                output.flush()?;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessQueryRow {
    pub id: String,
    pub generation: Option<u32>,
    pub states: Vec<String>,
    pub restart_policy: Option<String>,
    pub restart_action: Option<String>,
    pub ready: bool,
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessQueryReport {
    pub schema: String,
    pub classification: String,
    pub evidence_eligible: bool,
    pub components: Vec<ReadinessQueryRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadinessAck {
    schema: String,
    op: String,
    id: String,
    generation: u32,
    evidence_eligible: bool,
}

/// A service's separate, non-evidence channel to the currently running live daemon.
/// Constructing this publishes `started`; `ready` requires an actual successful
/// service request. Losing this connection retracts the active observation.
pub struct ReadinessPublisher {
    stream: BufReader<UnixStream>,
    component: ReadinessComponent,
    acknowledged: bool,
}

impl ReadinessPublisher {
    pub fn start_service(
        socket: &Path,
        id: String,
        restart_policy: crunch_service_readiness_core::RestartPolicy,
    ) -> io::Result<Self> {
        let stream = UnixStream::connect(socket)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut publisher = Self {
            stream: BufReader::new(stream),
            component: ReadinessComponent {
                id,
                kind: "service".to_owned(),
                depends_on: Vec::new(),
                user_states: Vec::new(),
                restart_policy: Some(restart_policy.as_str().to_owned()),
                stage_requirements: None,
            },
            acknowledged: false,
        };
        publisher.publish(vec!["started".to_owned()], None, false)?;
        Ok(publisher)
    }

    pub fn ready_after_successful_request(&mut self) -> io::Result<()> {
        self.publish(vec!["started".to_owned(), "ready".to_owned()], None, true)?;
        self.acknowledged = true;
        Ok(())
    }

    pub fn exited(&mut self, normal: bool) -> io::Result<()> {
        let state = if normal && self.acknowledged {
            "complete"
        } else {
            "failed"
        };
        let exit = if normal { "normal" } else { "abnormal" };
        self.publish(vec![state.to_owned()], Some(exit.to_owned()), self.acknowledged)
    }

    fn publish(&mut self, states: Vec<String>, exit: Option<String>, acknowledged: bool) -> io::Result<()> {
        let observation = ReadinessObservation {
            id: self.component.id.clone(),
            generation: 1,
            states,
            exit,
            request_acknowledged: acknowledged,
            proof_completion: None,
        };
        let request = ReadinessPublishFrame {
            schema: READINESS_SCHEMA,
            op: "publish",
            component: &self.component,
            observation: &observation,
        };
        serde_json::to_writer(self.stream.get_mut(), &request).map_err(io::Error::other)?;
        self.stream.get_mut().write_all(b"\n")?;
        self.stream.get_mut().flush()?;
        let reply = read_readiness_reply(&mut self.stream)?;
        let ack: ReadinessAck = serde_json::from_slice(&reply).map_err(io::Error::other)?;
        if ack.schema != READINESS_SCHEMA
            || ack.op != "ack"
            || ack.id != self.component.id
            || ack.generation != 1
            || ack.evidence_eligible
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "readiness-publish-not-acknowledged"));
        }
        Ok(())
    }
}

/// Typed opt-in coordination query; this never reads or modifies evidence.
pub fn query_readiness(socket: &Path) -> io::Result<ReadinessQueryReport> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let request = ReadinessQueryFrame {
        schema: READINESS_SCHEMA,
        op: "query",
        components: [],
    };
    serde_json::to_writer(&mut stream, &request).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    let reply = read_readiness_reply(&mut BufReader::new(stream))?;
    let report: ReadinessQueryReport = serde_json::from_slice(&reply).map_err(io::Error::other)?;
    if report.schema != READINESS_SCHEMA
        || report.classification != crunch_service_readiness_core::COORDINATION_CLASSIFICATION
        || report.evidence_eligible
    {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid-readiness-coordination-report"));
    }
    Ok(report)
}

fn read_readiness_reply(reader: &mut impl BufRead) -> io::Result<Vec<u8>> {
    const MAX_REPLY_BYTES: usize = 2_097_152;
    let mut reply = Vec::with_capacity(256);
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "readiness-reply-truncated"));
        }
        let end = available.iter().position(|byte| *byte == b'\n');
        let length = end.unwrap_or(available.len());
        if reply.len().saturating_add(length) > MAX_REPLY_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "readiness-reply-too-large"));
        }
        reply.extend_from_slice(&available[..length]);
        reader.consume(length + usize::from(end.is_some()));
        if end.is_some() {
            return Ok(reply);
        }
    }
}

fn probe_subscription_snapshot(socket: &Path) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let filter = LiveFilter {
        owner_run_ids: Vec::new(),
        job_ids: Vec::new(),
        kinds: Vec::new(),
    };
    let request = SubscriptionRequest {
        schema: LIVE_PROTOCOL_SCHEMA,
        op: "subscribe",
        filter: &filter,
    };
    serde_json::to_writer(&mut stream, &request).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    let mut reader = BufReader::new(stream);
    let beginning: serde_json::Value =
        serde_json::from_slice(&read_readiness_reply(&mut reader)?).map_err(io::Error::other)?;
    if beginning["schema"] != LIVE_PROTOCOL_SCHEMA || beginning["op"] != "snapshot-start" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "live-subscription-snapshot-start-missing"));
    }
    for _ in 0..=MAX_LIVE_FACTS {
        let event: serde_json::Value =
            serde_json::from_slice(&read_readiness_reply(&mut reader)?).map_err(io::Error::other)?;
        if event["schema"] != LIVE_PROTOCOL_SCHEMA {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "live-subscription-schema-mismatch"));
        }
        if event["op"] == "snapshot-end" {
            return Ok(());
        }
        if event["op"] != "publish" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "live-subscription-snapshot-incomplete"));
        }
    }
    Err(io::Error::new(io::ErrorKind::InvalidData, "live-subscription-snapshot-overflow"))
}

fn report_daemon_readiness(socket: PathBuf, done: &AtomicBool, normal_exit: &AtomicBool) {
    let id = format!("live-daemon-{}", std::process::id());
    let Ok(mut publisher) =
        ReadinessPublisher::start_service(&socket, id, crunch_service_readiness_core::RestartPolicy::Never)
    else {
        return;
    };
    if probe_subscription_snapshot(&socket).is_ok() {
        let _ = publisher.ready_after_successful_request();
    }
    while !done.load(Ordering::Acquire) {
        thread::sleep(IO_INTERVAL);
    }
    let _ = publisher.exited(normal_exit.load(Ordering::Acquire));
}

/// Serve transient facts until `stop` is set. A stale socket left by a crashed
/// daemon is removed only when owned by this user and refusing connections.
/// Invalid requests close their connection; no acknowledgement is sent to publishers.
/// Subscribers receive snapshot-start, matching publishes, snapshot-end, then diffs.
/// Empty filter arrays are wildcards; a job filter excludes worker-presence facts.
/// All active facts are lost when the daemon exits or restarts.
pub fn serve(socket_path: &Path, stop: &AtomicBool) -> io::Result<()> {
    let listener = bind_socket(socket_path)?;
    let metadata = fs::symlink_metadata(socket_path)?;
    let _cleanup = SocketCleanup {
        path: socket_path.to_owned(),
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    let state = Arc::new(Mutex::new(State::default()));
    let _ = writeln!(io::stderr().lock(), "live build daemon listening on {}", socket_path.display());
    let stopping = Arc::new(AtomicBool::new(false));
    let readiness_done = Arc::new(AtomicBool::new(false));
    let normal_exit = Arc::new(AtomicBool::new(false));
    let readiness_thread = {
        let socket = socket_path.to_owned();
        let done = Arc::clone(&readiness_done);
        let normal_exit = Arc::clone(&normal_exit);
        thread::spawn(move || report_daemon_readiness(socket, &done, &normal_exit))
    };
    let mut accept_error = None;
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut guard = lock_state(&state);
                if guard.connections >= MAX_CONNECTIONS {
                    continue;
                }
                guard.connections += 1;
                guard.next_client += 1;
                let client = guard.next_client;
                drop(guard);
                let state = Arc::clone(&state);
                let stopping = Arc::clone(&stopping);
                thread::spawn(move || {
                    handle_client(stream, &state, client, &stopping);
                    lock_state(&state).connections -= 1;
                });
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => thread::sleep(IO_INTERVAL),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                accept_error = Some(error);
                break;
            }
        }
    }
    let mut guard = lock_state(&state);
    guard.shutting_down = true;
    let owners: Vec<_> = guard.owners.iter().map(|(owner, client)| (owner.clone(), *client)).collect();
    for (owner, client) in owners {
        guard.close_owner(&owner, client);
    }
    guard.subscribers.clear(); // Writers drain queued retractions before closing.
    drop(guard);
    normal_exit.store(accept_error.is_none(), Ordering::Release);
    readiness_done.store(true, Ordering::Release);
    let _ = readiness_thread.join();
    stopping.store(true, Ordering::Release);
    while lock_state(&state).connections != 0 {
        thread::sleep(IO_INTERVAL);
    }
    accept_error.map_or(Ok(()), Err)
}

fn handle_client(mut stream: UnixStream, shared: &Arc<Mutex<State>>, client: u64, stop: &AtomicBool) {
    if stream.set_read_timeout(Some(IO_INTERVAL)).is_err() {
        return;
    }
    let mut owner: Option<String> = None;
    let mut readiness_owner: Option<String> = None;
    let mut buffer = Vec::with_capacity(MAX_FRAME_BYTES);
    let mut chunk = [0_u8; 1024];
    'read: while !stop.load(Ordering::Acquire) {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => {
                for byte in &chunk[..count] {
                    if *byte != b'\n' {
                        if buffer.len() == MAX_FRAME_BYTES {
                            break 'read;
                        }
                        buffer.push(*byte);
                        continue;
                    }
                    let request = match serde_json::from_slice::<Request>(&buffer) {
                        Ok(request) if request.schema() == LIVE_PROTOCOL_SCHEMA && readiness_owner.is_none() => request,
                        _ => {
                            if owner.is_some() {
                                break 'read;
                            }
                            let request = match serde_json::from_slice::<ReadinessRequest>(&buffer) {
                                Ok(request) if request.schema() == READINESS_SCHEMA => request,
                                _ => break 'read,
                            };
                            buffer.clear();
                            if stop.load(Ordering::Acquire)
                                || handle_readiness_request(&mut stream, shared, client, &mut readiness_owner, request)
                                    .is_err()
                            {
                                break 'read;
                            }
                            continue;
                        }
                    };
                    buffer.clear();
                    if stop.load(Ordering::Acquire) {
                        break 'read;
                    }
                    match request {
                        Request::Open { owner_run_id, .. }
                            if owner.is_none() && admit_identity(&owner_run_id).is_ok() =>
                        {
                            let mut state = lock_state(shared);
                            if state.shutting_down || state.owners.contains_key(&owner_run_id) {
                                break 'read;
                            }
                            state.owners.insert(owner_run_id.clone(), client);
                            owner = Some(owner_run_id);
                        }
                        Request::Publish { fact_id, fact, .. } if owner.is_some() => {
                            let owner_id = owner.as_ref().unwrap();
                            if fact.schema != LIVE_BUILD_FACT_SCHEMA
                                || !valid_fact_value(owner_id, &fact.value)
                                || fact_identity(owner_id, &fact.value).as_deref() != Ok(fact_id.as_str())
                            {
                                break 'read;
                            }
                            let fact = LiveBuildFact {
                                schema: LIVE_BUILD_FACT_SCHEMA,
                                value: fact.value,
                            };
                            if admit_fact_size(owner_id, &fact_id, &fact).is_err() {
                                break 'read;
                            }
                            let mut state = lock_state(shared);
                            if state.shutting_down {
                                break 'read;
                            }
                            if let Some(existing) = state.facts.get(&fact_id) {
                                if existing.owner != *owner_id {
                                    break 'read;
                                }
                                if existing.fact == fact {
                                    continue;
                                }
                            } else if state.facts.len() == MAX_LIVE_FACTS {
                                break 'read;
                            }
                            state.facts.insert(fact_id.clone(), Entry {
                                owner: owner_id.clone(),
                                fact: fact.clone(),
                            });
                            state.broadcast(
                                Event::Publish {
                                    schema: LIVE_PROTOCOL_SCHEMA,
                                    owner_run_id: owner_id.clone(),
                                    fact_id,
                                    fact: fact.clone(),
                                },
                                owner_id,
                                &fact.value,
                            );
                        }
                        Request::Retract { fact_id, .. } if owner.is_some() => {
                            let owner_id = owner.as_ref().unwrap();
                            let mut state = lock_state(shared);
                            if state.shutting_down
                                || state.facts.get(&fact_id).is_none_or(|entry| entry.owner != *owner_id)
                            {
                                break 'read;
                            }
                            state.retract(owner_id, &fact_id);
                        }
                        Request::Stop { .. } if owner.is_some() => break 'read,
                        Request::Subscribe { filter, .. } if owner.is_none() => {
                            let Some(filter) = Filter::parse(filter) else {
                                break 'read;
                            };
                            serve_subscriber(stream, shared, client, filter);
                            return;
                        }
                        _ => break 'read,
                    }
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                ) =>
            {
                continue;
            }
            Err(_) => break,
        }
    }
    if let Some(owner) = owner {
        lock_state(shared).close_owner(&owner, client);
    }
    if readiness_owner.is_some() {
        close_readiness_owner(shared, client);
    }
}

type ReadinessView<'a> = (&'a ReadinessComponent, Option<&'a ReadinessObservation>);

fn readiness_views<'a>(
    state: &'a State,
    supplied: &'a [ReadinessComponent],
    replacement: Option<(&'a ReadinessComponent, &'a ReadinessObservation)>,
    excluded: Option<&str>,
) -> Result<Vec<ReadinessView<'a>>, String> {
    let mut views = state
        .readiness_records
        .iter()
        .filter(|(id, _)| Some(id.as_str()) != excluded)
        .map(|(_, record)| (&record.component, record.observation.as_ref()))
        .collect::<Vec<_>>();
    for (index, component) in supplied.iter().enumerate() {
        if supplied[..index].iter().any(|earlier| earlier.id == component.id) {
            return Err("duplicate-readiness-component".into());
        }
        if let Some((existing, _)) = views.iter().find(|(existing, _)| existing.id == component.id) {
            if *existing != component {
                return Err("readiness-declaration-drift".into());
            }
        } else {
            views.push((component, None));
        }
    }
    if let Some((component, observation)) = replacement {
        let Some(entry) = views.iter_mut().find(|(existing, _)| existing.id == component.id) else {
            return Err("missing-readiness-component".into());
        };
        entry.1 = Some(observation);
    }
    if views.len() > crunch_service_readiness_core::MAX_COMPONENTS as usize {
        return Err("too-many-readiness-components".into());
    }
    views.sort_unstable_by(|left, right| left.0.id.cmp(&right.0.id));
    Ok(views)
}

fn with_readiness_snapshot<R>(
    views: &[ReadinessView<'_>],
    project: impl FnOnce(&crunch_service_readiness_core::Snapshot<'_>) -> Result<R, String>,
) -> Result<R, String> {
    use crunch_service_readiness_core as core;

    let dependencies = views
        .iter()
        .map(|(component, _)| component.depends_on.iter().map(String::as_str).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let user_states = views
        .iter()
        .map(|(component, _)| component.user_states.iter().map(String::as_str).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let states = views
        .iter()
        .map(|(_, observation)| {
            observation
                .map(|value| value.states.iter().map(String::as_str).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let mut components = Vec::with_capacity(views.len());
    let mut observations = Vec::with_capacity(views.len());
    for (index, (component, observation)) in views.iter().enumerate() {
        let kind = match component.kind.as_str() {
            "service" => core::ComponentKind::Service,
            "proof-stage" => core::ComponentKind::ProofStage,
            _ => return Err("unknown-readiness-component-kind".into()),
        };
        let restart_policy = component
            .restart_policy
            .as_deref()
            .map(|value| core::RestartPolicy::parse(value).ok_or("unknown-readiness-restart-policy"))
            .transpose()?;
        components.push(core::Component {
            id: &component.id,
            kind,
            depends_on: &dependencies[index],
            user_states: &user_states[index],
            restart_policy,
            stage_requirements: component.stage_requirements.as_ref().map(|value| core::StageRequirements {
                require_action_reconciliation: value.require_action_reconciliation,
                require_v2_receipt: value.require_v2_receipt,
            }),
        });
        if let Some(observation) = observation {
            let exit = match observation.exit.as_deref() {
                None => None,
                Some("normal") => Some(core::ExitKind::Normal),
                Some("abnormal") => Some(core::ExitKind::Abnormal),
                Some(_) => return Err("unknown-readiness-exit".into()),
            };
            observations.push(core::Observation {
                id: &observation.id,
                generation: observation.generation,
                states: &states[index],
                exit,
                request_acknowledged: observation.request_acknowledged,
                proof_completion: observation.proof_completion.as_ref().map(|value| core::ProofCompletion {
                    stage_evidence_digest_blake3: &value.stage_evidence_digest_blake3,
                    output_digest_blake3: &value.output_digest_blake3,
                    execution_verified: value.execution_verified,
                    action_reconciled: value.action_reconciled,
                    v2_receipt_verified: value.v2_receipt_verified,
                }),
            });
        }
    }
    project(&core::Snapshot {
        schema: READINESS_SCHEMA,
        components: &components,
        observations: &observations,
    })
}

fn readiness_report_json(report: &crunch_service_readiness_core::ReadinessReport<'_>) -> serde_json::Value {
    let components = report
        .components
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "generation": row.generation,
                "states": &row.states,
                "restart_policy": row.restart_policy.map(crunch_service_readiness_core::RestartPolicy::as_str),
                "restart_action": row.restart_action.map(crunch_service_readiness_core::RestartAction::as_str),
                "ready": row.ready,
                "blocked_by": &row.blocked_by,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema": report.schema,
        "classification": report.classification,
        "evidence_eligible": report.evidence_eligible,
        "components": components,
    })
}

fn write_readiness_reply(stream: &mut UnixStream, reply: &serde_json::Value) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    serde_json::to_writer(&mut *stream, reply).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn handle_readiness_request(
    stream: &mut UnixStream,
    shared: &Arc<Mutex<State>>,
    client: u64,
    owner: &mut Option<String>,
    request: ReadinessRequest,
) -> io::Result<()> {
    use crunch_service_readiness_core as core;

    let reply = match request {
        ReadinessRequest::Query { components, .. } if owner.is_none() => {
            let state = lock_state(shared);
            let views = readiness_views(&state, &components, None, None).map_err(io::Error::other)?;
            with_readiness_snapshot(&views, |snapshot| {
                core::evaluate(snapshot)
                    .map(|report| readiness_report_json(&report))
                    .map_err(|error| format!("readiness-query-rejected:{error:?}"))
            })
            .map_err(io::Error::other)?
        }
        ReadinessRequest::Publish {
            component, observation, ..
        } if owner.as_ref().is_none_or(|id| *id == component.id) => {
            if component.id != observation.id {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "readiness-observation-identity-mismatch"));
            }
            let mut state = lock_state(shared);
            let owned_terminal_exit = state.shutting_down
                && owner.as_deref() == Some(component.id.as_str())
                && state.readiness_records.get(&component.id).is_some_and(|record| {
                    record.owner == Some(client)
                        && record.component == component
                        && record.observation.as_ref().is_some_and(|previous| {
                            previous.id == observation.id
                                && previous.generation == observation.generation
                                && previous.exit.is_none()
                        })
                        && matches!(observation.exit.as_deref(), Some("normal" | "abnormal"))
                        && matches!(observation.states.as_slice(), [state] if state == "complete" || state == "failed")
                });
            if (state.shutting_down && !owned_terminal_exit)
                || state
                    .readiness_records
                    .get(&component.id)
                    .is_some_and(|record| record.owner.is_some_and(|other| other != client))
            {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "readiness-component-owned-by-other-client",
                ));
            }
            let eviction = if !state.readiness_records.contains_key(&component.id)
                && state.readiness_records.len() >= core::MAX_COMPONENTS as usize
            {
                Some(
                    state
                        .readiness_records
                        .iter()
                        .filter(|(_, record)| record.owner.is_none())
                        .min_by_key(|(_, record)| record.sequence)
                        .map(|(id, _)| id.clone())
                        .ok_or_else(|| io::Error::other("readiness-component-capacity-exhausted"))?,
                )
            } else {
                None
            };
            let supplied = std::slice::from_ref(&component);
            let before = readiness_views(&state, supplied, None, eviction.as_deref()).map_err(io::Error::other)?;
            let after = readiness_views(&state, supplied, Some((&component, &observation)), eviction.as_deref())
                .map_err(io::Error::other)?;
            with_readiness_snapshot(&before, |previous| {
                let previous =
                    core::evaluate(previous).map_err(|error| format!("readiness-previous-rejected:{error:?}"))?;
                with_readiness_snapshot(&after, |current| {
                    core::advance(&previous, current)
                        .map(|_| ())
                        .map_err(|error| format!("readiness-transition-rejected:{error:?}"))
                })
            })
            .map_err(io::Error::other)?;
            if let Some(id) = eviction {
                state.readiness_records.remove(&id);
            }
            state.next_readiness_sequence = state
                .next_readiness_sequence
                .checked_add(1)
                .ok_or_else(|| io::Error::other("readiness-sequence-exhausted"))?;
            let sequence = state.next_readiness_sequence;
            *owner = Some(component.id.clone());
            let id = component.id.clone();
            let generation = observation.generation;
            state.readiness_records.insert(id.clone(), ReadinessRecord {
                component,
                observation: Some(observation),
                owner: Some(client),
                sequence,
            });
            serde_json::json!({
                "schema": READINESS_SCHEMA,
                "op": "ack",
                "id": id,
                "generation": generation,
                "evidence_eligible": false,
            })
        }
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "readiness-request-out-of-sequence")),
    };
    write_readiness_reply(stream, &reply)
}

fn close_readiness_owner(shared: &Arc<Mutex<State>>, client: u64) {
    let mut state = lock_state(shared);
    for record in state.readiness_records.values_mut() {
        if record.owner == Some(client) {
            record.owner = None;
            if record.observation.as_ref().is_some_and(|observation| observation.exit.is_none()) {
                record.observation = None;
            }
        }
    }
}

fn serve_subscriber(mut stream: UnixStream, shared: &Arc<Mutex<State>>, client: u64, filter: Filter) {
    let (sender, receiver) = mpsc::sync_channel(SUBSCRIBER_QUEUE);
    let alive = Arc::new(AtomicBool::new(true));
    let snapshot = {
        let mut state = lock_state(shared);
        if state.shutting_down || state.subscribers.len() >= MAX_SUBSCRIBERS {
            return;
        }
        let snapshot: Vec<Event> = state
            .facts
            .iter()
            .filter(|(_, entry)| filter.matches(&entry.owner, &entry.fact.value))
            .map(|(id, entry)| Event::Publish {
                schema: LIVE_PROTOCOL_SCHEMA,
                owner_run_id: entry.owner.clone(),
                fact_id: id.clone(),
                fact: entry.fact.clone(),
            })
            .collect();
        state.subscribers.insert(client, Subscriber {
            filter,
            sender,
            alive: Arc::clone(&alive),
        });
        snapshot
    };
    let writer_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(_) => {
            lock_state(shared).subscribers.remove(&client);
            return;
        }
    };
    let writer_alive = Arc::clone(&alive);
    let writer = thread::spawn(move || write_subscriber(writer_stream, receiver, snapshot, &writer_alive));
    let mut byte = [0_u8; 1];
    while alive.load(Ordering::Acquire) {
        match stream.read(&mut byte) {
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                ) =>
            {
                continue;
            }
            _ => break, // Subscribers only send their initial subscription frame.
        }
    }
    alive.store(false, Ordering::Release);
    lock_state(shared).subscribers.remove(&client);
    let _ = stream.shutdown(Shutdown::Both);
    let _ = writer.join();
}

fn write_subscriber(mut stream: UnixStream, receiver: Receiver<Event>, snapshot: Vec<Event>, alive: &AtomicBool) {
    if stream.set_write_timeout(Some(IO_INTERVAL)).is_err() {
        alive.store(false, Ordering::Release);
        return;
    }
    let beginning = Event::SnapshotStart {
        schema: LIVE_PROTOCOL_SCHEMA,
    };
    let end = Event::SnapshotEnd {
        schema: LIVE_PROTOCOL_SCHEMA,
    };
    for event in std::iter::once(&beginning).chain(snapshot.iter()).chain(std::iter::once(&end)) {
        if !alive.load(Ordering::Acquire) || write_event(&mut stream, event).is_err() {
            alive.store(false, Ordering::Release);
            return;
        }
    }
    while alive.load(Ordering::Acquire) {
        match receiver.recv_timeout(IO_INTERVAL) {
            Ok(event) if write_event(&mut stream, &event).is_err() => break,
            Ok(_) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    alive.store(false, Ordering::Release);
    let _ = stream.shutdown(Shutdown::Both);
}

fn write_event(stream: &mut UnixStream, event: &Event) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, event).map_err(io::Error::other)?;
    stream.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use std::io::BufRead;
    use std::io::BufReader;
    use std::os::fd::AsRawFd;
    use std::sync::atomic::AtomicBool;

    use serde_json::Value;
    use serde_json::json;

    use super::*;

    struct Fixture {
        path: PathBuf,
        stop: Arc<AtomicBool>,
        thread: Option<thread::JoinHandle<io::Result<()>>>,
    }

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mantle-live-daemon-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            Self::at_path(path)
        }

        fn at_path(path: PathBuf) -> Self {
            let stop = Arc::new(AtomicBool::new(false));
            let flag = Arc::clone(&stop);
            let socket = path.clone();
            let thread = thread::spawn(move || serve(&socket, &flag));
            for _ in 0..100 {
                if fs::metadata(&path).is_ok_and(|meta| meta.permissions().mode() & 0o777 == 0o600)
                    && UnixStream::connect(&path).is_ok()
                {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            assert!(UnixStream::connect(&path).is_ok(), "daemon did not start");
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
            Self {
                path,
                stop,
                thread: Some(thread),
            }
        }

        fn connect(&self) -> UnixStream {
            let stream = UnixStream::connect(&self.path).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            stream
        }

        fn shutdown(&mut self) {
            if let Some(thread) = self.thread.take() {
                self.stop.store(true, Ordering::Release);
                thread.join().unwrap().unwrap();
                assert!(!self.path.exists());
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            self.shutdown();
        }
    }

    fn send(stream: &mut UnixStream, message: Value) {
        stream.write_all(format!("{message}\n").as_bytes()).unwrap();
    }

    fn read(reader: &mut BufReader<UnixStream>) -> Value {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(!line.is_empty(), "subscriber disconnected");
        serde_json::from_str(&line).unwrap()
    }

    fn subscribe(fixture: &Fixture, owners: Vec<&str>, jobs: Vec<&str>, kinds: Vec<&str>) -> BufReader<UnixStream> {
        let mut stream = fixture.connect();
        send(
            &mut stream,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "subscribe", "filter": {"owner_run_ids": owners, "job_ids": jobs, "kinds": kinds}}),
        );
        let mut reader = BufReader::new(stream);
        assert_eq!(read(&mut reader)["op"], "snapshot-start");
        reader
    }

    fn publish(stream: &mut UnixStream, owner: &str, value: LiveBuildFactValue) -> String {
        let id = fact_identity(owner, &value).unwrap();
        send(
            stream,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": value}}),
        );
        id
    }

    fn goal(job: &str) -> LiveBuildFactValue {
        LiveBuildFactValue::Goal {
            job_id: job.into(),
            phase: super::super::LiveGoalPhase::Discovered,
            worker_endpoint_id: None,
            attempt_id: None,
            fence_generation: None,
        }
    }

    #[test]
    fn readiness_subscription_ack_precedes_ready_and_lost_publisher_retracts_it() {
        let fixture = Fixture::new();
        let daemon_id = format!("live-daemon-{}", std::process::id());
        let daemon_ready = (0..100).any(|_| {
            let ready = query_readiness(&fixture.path)
                .unwrap()
                .components
                .iter()
                .any(|row| row.id == daemon_id && row.ready && row.restart_policy.as_deref() == Some("never"));
            if !ready {
                thread::sleep(Duration::from_millis(10));
            }
            ready
        });
        assert!(daemon_ready, "same-daemon subscriber snapshot must precede readiness");

        let mut publisher = ReadinessPublisher::start_service(
            &fixture.path,
            "cache-test".to_owned(),
            crunch_service_readiness_core::RestartPolicy::Never,
        )
        .unwrap();
        let started = query_readiness(&fixture.path).unwrap();
        assert!(!started.evidence_eligible);
        let row = started.components.iter().find(|row| row.id == "cache-test").unwrap();
        assert_eq!(row.states, ["started"]);
        assert!(!row.ready);

        let mut forged = fixture.connect();
        send(
            &mut forged,
            json!({
                "schema": READINESS_SCHEMA,
                "op": "publish",
                "component": {
                    "id": "forged",
                    "kind": "service",
                    "depends_on": [],
                    "user_states": [],
                    "restart_policy": "never",
                    "stage_requirements": null
                },
                "observation": {
                    "id": "forged",
                    "generation": 1,
                    "states": ["started", "ready"],
                    "exit": null,
                    "request_acknowledged": true,
                    "proof_completion": null
                }
            }),
        );
        assert_eq!(forged.read(&mut [0_u8; 1]).unwrap(), 0, "ready cannot be the first observed state");
        assert!(query_readiness(&fixture.path).unwrap().components.iter().all(|row| row.id != "forged"));

        publisher.ready_after_successful_request().unwrap();
        let ready = query_readiness(&fixture.path).unwrap();
        let row = ready.components.iter().find(|row| row.id == "cache-test").unwrap();
        assert!(row.ready);
        assert_eq!(row.states, ["started", "ready"]);
        drop(publisher);
        let retracted = (0..100).any(|_| {
            let report = query_readiness(&fixture.path).unwrap();
            let row = report.components.iter().find(|row| row.id == "cache-test").unwrap();
            if row.ready {
                thread::sleep(Duration::from_millis(10));
            }
            !row.ready && row.generation.is_none()
        });
        assert!(retracted, "a dropped readiness publisher must not remain ready");
    }

    #[test]
    fn readiness_abnormal_exit_is_terminal_and_disallows_never_restart() {
        let fixture = Fixture::new();
        let mut publisher = ReadinessPublisher::start_service(
            &fixture.path,
            "remote-test".to_owned(),
            crunch_service_readiness_core::RestartPolicy::Never,
        )
        .unwrap();
        publisher.ready_after_successful_request().unwrap();
        publisher.exited(false).unwrap();
        drop(publisher);
        let report = query_readiness(&fixture.path).unwrap();
        let row = report.components.iter().find(|row| row.id == "remote-test").unwrap();
        assert_eq!(row.states, ["failed"]);
        assert_eq!(row.restart_action.as_deref(), Some("none"));
        assert!(!row.ready);
        let mut early = ReadinessPublisher::start_service(
            &fixture.path,
            "remote-early".to_owned(),
            crunch_service_readiness_core::RestartPolicy::Never,
        )
        .unwrap();
        early.exited(false).unwrap();
        drop(early);
        let early_report = query_readiness(&fixture.path).unwrap();
        let early_row = early_report.components.iter().find(|row| row.id == "remote-early").unwrap();
        assert_eq!(early_row.states, ["failed"]);
        assert!(!early_row.ready, "a service exiting before any acknowledged request cannot become ready");
        assert!(
            ReadinessPublisher::start_service(
                &fixture.path,
                "remote-test".to_owned(),
                crunch_service_readiness_core::RestartPolicy::Never
            )
            .is_err(),
            "a new process uses a new invocation ID; this daemon never auto-restarts the old one"
        );
    }

    #[test]
    fn manual_daemon_relaunch_starts_an_empty_never_policy_epoch() {
        let mut fixture = Fixture::new();
        let previous = ReadinessPublisher::start_service(
            &fixture.path,
            "previous-invocation".to_owned(),
            crunch_service_readiness_core::RestartPolicy::Never,
        )
        .unwrap();
        let path = fixture.path.clone();
        fixture.shutdown();
        drop(previous);
        let relaunched = Fixture::at_path(path);
        let daemon_id = format!("live-daemon-{}", std::process::id());
        let fresh = (0..100).any(|_| {
            let report = query_readiness(&relaunched.path).unwrap();
            assert!(report.components.iter().all(|row| row.id != "previous-invocation"));
            let current = report.components.iter().find(|row| row.id == daemon_id);
            let ready = current.is_some_and(|row| {
                row.ready && row.generation == Some(1) && row.restart_policy.as_deref() == Some("never")
            });
            if !ready {
                thread::sleep(Duration::from_millis(10));
            }
            ready
        });
        assert!(fresh, "manual relaunch must re-probe the new empty daemon, never inherit prior readiness");
    }

    #[test]
    fn dependent_socket_start_requires_upstream_request_and_reblocks_on_loss() {
        let fixture = Fixture::new();
        let mut upstream = ReadinessPublisher::start_service(
            &fixture.path,
            "upstream".to_owned(),
            crunch_service_readiness_core::RestartPolicy::Never,
        )
        .unwrap();
        let dependent_start = json!({
            "schema": READINESS_SCHEMA,
            "op": "publish",
            "component": {
                "id": "dependent",
                "kind": "service",
                "depends_on": ["upstream"],
                "user_states": [],
                "restart_policy": "never",
                "stage_requirements": null
            },
            "observation": {
                "id": "dependent",
                "generation": 1,
                "states": ["started"],
                "exit": null,
                "request_acknowledged": false,
                "proof_completion": null
            }
        });
        let mut blocked = fixture.connect();
        send(&mut blocked, dependent_start.clone());
        assert_eq!(blocked.read(&mut [0_u8; 1]).unwrap(), 0, "a started upstream does not unblock a new dependent");
        assert!(query_readiness(&fixture.path).unwrap().components.iter().all(|row| row.id != "dependent"));

        upstream.ready_after_successful_request().unwrap();
        let mut dependent = BufReader::new(fixture.connect());
        send(dependent.get_mut(), dependent_start);
        let ack: Value = serde_json::from_slice(&read_readiness_reply(&mut dependent).unwrap()).unwrap();
        assert_eq!(ack["op"], "ack");
        let report = query_readiness(&fixture.path).unwrap();
        let row = report.components.iter().find(|row| row.id == "dependent").unwrap();
        assert_eq!(row.states, ["started"]);
        assert!(row.blocked_by.is_empty());

        drop(upstream);
        let reblocked = (0..100).any(|_| {
            let report = query_readiness(&fixture.path).unwrap();
            let row = report.components.iter().find(|row| row.id == "dependent").unwrap();
            if row.blocked_by.is_empty() {
                thread::sleep(Duration::from_millis(10));
            }
            row.blocked_by == ["upstream"]
        });
        assert!(reblocked, "dependent must name its lost predecessor");
    }

    #[test]
    fn readiness_frames_never_enter_existing_live_subscriber_stream() {
        let fixture = Fixture::new();
        let mut subscriber = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut subscriber)["op"], "snapshot-end");
        let mut readiness = ReadinessPublisher::start_service(
            &fixture.path,
            "separate-service".to_owned(),
            crunch_service_readiness_core::RestartPolicy::Never,
        )
        .unwrap();
        readiness.ready_after_successful_request().unwrap();
        assert!(
            query_readiness(&fixture.path)
                .unwrap()
                .components
                .iter()
                .any(|row| { row.id == "separate-service" && row.ready })
        );
        readiness.exited(false).unwrap();
        subscriber.get_ref().set_read_timeout(Some(Duration::from_millis(150))).unwrap();
        let mut line = String::new();
        assert!(matches!(
            subscriber.read_line(&mut line),
            Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
        ));
        assert!(line.is_empty(), "readiness must never publish into live subscriber snapshots or diffs");
        subscriber.get_ref().set_read_timeout(Some(Duration::from_secs(2))).unwrap();

        let mut live = fixture.connect();
        send(&mut live, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "real-build"}));
        let id = publish(&mut live, "real-build", goal("real-goal"));
        let published = read(&mut subscriber);
        assert_eq!(published["op"], "publish");
        assert_eq!(published["fact_id"], id);
        send(&mut live, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "retract", "fact_id": id}));
        let retracted = read(&mut subscriber);
        assert_eq!(retracted["op"], "retract");
        assert_eq!(retracted["fact_id"], id);
    }

    #[test]
    fn multi_root_snapshot_then_independent_terminal_transitions_keep_unrelated_facts() {
        let fixture = Fixture::new();
        let mut build = fixture.connect();
        let mut unrelated = fixture.connect();
        send(&mut build, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "build"}));
        send(&mut unrelated, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "other"}));
        let mut barrier = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut barrier)["op"], "snapshot-end");
        let alpha = publish(&mut build, "build", goal("alpha"));
        assert_eq!(read(&mut barrier)["fact_id"], alpha);
        let beta = publish(&mut build, "build", goal("beta"));
        assert_eq!(read(&mut barrier)["fact_id"], beta);
        let gamma = publish(&mut build, "build", goal("gamma"));
        assert_eq!(read(&mut barrier)["fact_id"], gamma);
        let outsider = publish(&mut unrelated, "other", goal("alpha"));
        assert_eq!(read(&mut barrier)["fact_id"], outsider);

        let mut filtered =
            subscribe(&fixture, vec!["build"], vec!["alpha", "beta", "gamma"], vec!["goal", "terminal-outcome"]);
        let initial = [read(&mut filtered), read(&mut filtered), read(&mut filtered)];
        assert_eq!(
            BTreeSet::from([
                initial[0]["fact_id"].as_str().unwrap(),
                initial[1]["fact_id"].as_str().unwrap(),
                initial[2]["fact_id"].as_str().unwrap(),
            ]),
            BTreeSet::from([alpha.as_str(), beta.as_str(), gamma.as_str()])
        );
        assert_eq!(read(&mut filtered)["op"], "snapshot-end");
        let mut selected = subscribe(&fixture, vec!["build"], vec!["alpha", "beta"], vec!["goal", "terminal-outcome"]);
        let two_roots = [read(&mut selected), read(&mut selected)];
        assert_eq!(
            BTreeSet::from([
                two_roots[0]["fact_id"].as_str().unwrap(),
                two_roots[1]["fact_id"].as_str().unwrap()
            ]),
            BTreeSet::from([alpha.as_str(), beta.as_str()])
        );
        assert_eq!(read(&mut selected)["op"], "snapshot-end", "unrelated gamma and the other owner stay filtered");

        let terminal = |job: &str, outcome| LiveBuildFactValue::TerminalOutcome {
            job_id: job.to_string(),
            attempt_id: None,
            outcome,
            terminal_phase: Some(
                if outcome == LiveTerminalOutcome::Cancelled {
                    "coordination"
                } else {
                    "build"
                }
                .into(),
            ),
        };
        send(&mut build, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "retract", "fact_id": alpha}));
        assert_eq!(read(&mut filtered)["fact_id"], alpha);
        assert_eq!(read(&mut selected)["fact_id"], alpha);
        let alpha_done = publish(&mut build, "build", terminal("alpha", LiveTerminalOutcome::Succeeded));
        assert_eq!(read(&mut filtered)["fact_id"], alpha_done);
        assert_eq!(read(&mut selected)["fact_id"], alpha_done);

        let mut beta_only = subscribe(&fixture, vec!["build"], vec!["beta"], vec!["goal"]);
        assert_eq!(read(&mut beta_only)["fact_id"], beta, "unrelated root survives alpha completion");
        assert_eq!(read(&mut beta_only)["op"], "snapshot-end");
        let mut outside_only = subscribe(&fixture, vec!["other"], vec!["alpha"], vec!["goal"]);
        assert_eq!(read(&mut outside_only)["fact_id"], outsider);
        assert_eq!(read(&mut outside_only)["op"], "snapshot-end");

        send(&mut build, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "retract", "fact_id": beta}));
        assert_eq!(read(&mut filtered)["fact_id"], beta);
        assert_eq!(read(&mut beta_only)["op"], "retract");
        assert_eq!(read(&mut selected)["fact_id"], beta);
        let beta_failed = publish(&mut build, "build", terminal("beta", LiveTerminalOutcome::Failed));
        assert_eq!(read(&mut filtered)["fact_id"], beta_failed);
        assert_eq!(read(&mut selected)["fact_id"], beta_failed);
        send(&mut build, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "retract", "fact_id": gamma}));
        assert_eq!(read(&mut filtered)["fact_id"], gamma);
        let gamma_cancelled = publish(&mut build, "build", terminal("gamma", LiveTerminalOutcome::Cancelled));
        assert_eq!(read(&mut filtered)["fact_id"], gamma_cancelled);
        drop(build);
        let withdrawn = [read(&mut filtered), read(&mut filtered), read(&mut filtered)];
        assert!(withdrawn.iter().all(|event| event["op"] == "retract"));
        assert_eq!(
            BTreeSet::from([
                withdrawn[0]["fact_id"].as_str().unwrap(),
                withdrawn[1]["fact_id"].as_str().unwrap(),
                withdrawn[2]["fact_id"].as_str().unwrap(),
            ]),
            BTreeSet::from([alpha_done.as_str(), beta_failed.as_str(), gamma_cancelled.as_str()])
        );
        let selected_withdrawn = [read(&mut selected), read(&mut selected)];
        assert!(selected_withdrawn.iter().all(|event| event["op"] == "retract"));
        assert_eq!(
            BTreeSet::from([
                selected_withdrawn[0]["fact_id"].as_str().unwrap(),
                selected_withdrawn[1]["fact_id"].as_str().unwrap(),
            ]),
            BTreeSet::from([alpha_done.as_str(), beta_failed.as_str()]),
            "no gamma event may reach the two-root subscription"
        );
        let mut still_other = subscribe(&fixture, vec!["other"], vec!["alpha"], vec!["goal"]);
        assert_eq!(read(&mut still_other)["fact_id"], outsider);
        assert_eq!(read(&mut still_other)["op"], "snapshot-end");
    }

    #[test]
    fn owners_filters_and_snapshot_diffs() {
        let fixture = Fixture::new();
        let mut alpha = fixture.connect();
        let mut beta = fixture.connect();
        send(&mut alpha, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "root-a"}));
        send(&mut beta, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "root-b"}));
        let mut barrier = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut barrier)["op"], "snapshot-end");
        let a = publish(&mut alpha, "root-a", goal("job-a"));
        let b = publish(&mut beta, "root-b", goal("job-a"));
        let received = [read(&mut barrier), read(&mut barrier)];
        assert_eq!(
            BTreeSet::from([
                received[0]["fact_id"].as_str().unwrap(),
                received[1]["fact_id"].as_str().unwrap()
            ]),
            BTreeSet::from([a.as_str(), b.as_str()])
        );
        let mut all = subscribe(&fixture, vec![], vec![], vec![]);
        let first = read(&mut all);
        let second = read(&mut all);
        assert_eq!(first["op"], "publish");
        assert_eq!(second["op"], "publish");
        assert_eq!(
            BTreeSet::from([first["fact_id"].as_str().unwrap(), second["fact_id"].as_str().unwrap()]),
            BTreeSet::from([a.as_str(), b.as_str()])
        );
        assert_eq!(read(&mut all)["op"], "snapshot-end");
        let mut filtered = subscribe(&fixture, vec!["root-a"], vec!["job-a"], vec!["goal"]);
        assert_eq!(read(&mut filtered)["fact_id"], a);
        assert_eq!(read(&mut filtered)["op"], "snapshot-end");
        let other = publish(&mut beta, "root-b", goal("job-b"));
        assert_eq!(read(&mut all)["fact_id"], other);
        send(&mut alpha, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "retract", "fact_id": a}));
        assert_eq!(read(&mut all)["fact_id"], a);
        assert_eq!(read(&mut filtered)["op"], "retract");
        drop(beta);
        let lost = [read(&mut all), read(&mut all)];
        assert!(lost.iter().all(|event| event["op"] == "retract" && event["owner_run_id"] == "root-b"));
        assert_eq!(
            BTreeSet::from([
                lost[0]["fact_id"].as_str().unwrap(),
                lost[1]["fact_id"].as_str().unwrap()
            ]),
            BTreeSet::from([b.as_str(), other.as_str()])
        );
    }

    #[test]
    fn invalid_inputs_disconnect_owner_and_retract_only_its_facts() {
        let fixture = Fixture::new();
        let mut owner = fixture.connect();
        send(&mut owner, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "owner"}));
        let mut watch = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut watch)["op"], "snapshot-end");
        let id = publish(&mut owner, "owner", goal("job"));
        assert_eq!(read(&mut watch)["fact_id"], id);
        send(
            &mut owner,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": {"kind": "unknown"}}}),
        );
        assert_eq!(read(&mut watch)["op"], "retract");
        let mut restarted = fixture.connect();
        send(&mut restarted, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "owner"}));
        let again = publish(&mut restarted, "owner", goal("job"));
        assert_eq!(read(&mut watch)["fact_id"], again);
        send(&mut restarted, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "stop"}));
        assert_eq!(read(&mut watch)["op"], "retract");
        let mut malformed = fixture.connect();
        send(
            &mut malformed,
            json!({"schema": "wrong", "op": "subscribe", "filter": {"owner_run_ids": [], "job_ids": [], "kinds": []}}),
        );
        assert_eq!(malformed.read(&mut [0; 1]).unwrap(), 0);
        let mut oversized = fixture.connect();
        oversized.write_all(&vec![b'x'; MAX_FRAME_BYTES + 1]).unwrap();
        assert_eq!(oversized.read(&mut [0; 1]).unwrap(), 0);
        let mut invalid_filter = fixture.connect();
        send(
            &mut invalid_filter,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "subscribe", "filter": {"owner_run_ids": [], "job_ids": [], "kinds": ["not-a-kind"]}}),
        );
        assert_eq!(invalid_filter.read(&mut [0; 1]).unwrap(), 0);
        let mut unreviewed = fixture.connect();
        send(&mut unreviewed, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "unreviewed"}));
        let valid_id = fact_identity("unreviewed", &goal("job")).unwrap();
        send(
            &mut unreviewed,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": valid_id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": {"kind": "goal", "job_id": "job", "phase": "discovered", "worker_endpoint_id": null, "attempt_id": null, "fence_generation": null, "evidence": "trusted"}}}),
        );
        assert_eq!(unreviewed.read(&mut [0; 1]).unwrap(), 0);
        let mut too_many_filters = fixture.connect();
        let ids: Vec<_> = (0..MAX_FILTER_ITEMS + 1).map(|index| format!("job-{index}")).collect();
        send(
            &mut too_many_filters,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "subscribe", "filter": {"owner_run_ids": [], "job_ids": ids, "kinds": []}}),
        );
        assert_eq!(too_many_filters.read(&mut [0; 1]).unwrap(), 0);
        let mut invalid_identity = fixture.connect();
        send(
            &mut invalid_identity,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "x".repeat(super::super::MAX_LIVE_ID_BYTES + 1)}),
        );
        assert_eq!(invalid_identity.read(&mut [0; 1]).unwrap(), 0);
        let mut forged = fixture.connect();
        send(&mut forged, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "forged"}));
        send(
            &mut forged,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": "0".repeat(64), "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": goal("job")}}),
        );
        assert_eq!(forged.read(&mut [0; 1]).unwrap(), 0);
        let mut zero_generation = fixture.connect();
        send(
            &mut zero_generation,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "bad-worker"}),
        );
        let invalid = LiveBuildFactValue::WorkerPresence {
            endpoint_id: "worker".into(),
            generation: 0,
        };
        let invalid_id = fact_identity("bad-worker", &invalid).unwrap();
        send(
            &mut zero_generation,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": invalid_id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": invalid}}),
        );
        assert_eq!(zero_generation.read(&mut [0; 1]).unwrap(), 0);
        let mut empty = subscribe(&fixture, vec!["bad-worker"], vec![], vec![]);
        assert_eq!(read(&mut empty)["op"], "snapshot-end");
        let quoted = "\"".repeat(super::super::MAX_LIVE_ID_BYTES);
        let mut oversized_fact = fixture.connect();
        send(&mut oversized_fact, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": quoted}));
        let huge = LiveBuildFactValue::Goal {
            job_id: quoted.clone(),
            phase: LiveGoalPhase::Dispatched,
            worker_endpoint_id: Some(quoted.clone()),
            attempt_id: Some(quoted.clone()),
            fence_generation: Some(1),
        };
        let huge_id = fact_identity(&quoted, &huge).unwrap();
        send(
            &mut oversized_fact,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": huge_id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": huge}}),
        );
        assert_eq!(oversized_fact.read(&mut [0; 1]).unwrap(), 0);
    }

    #[test]
    fn worker_loss_retracts_presence_but_terminal_persists_until_owner_stops() {
        let fixture = Fixture::new();
        let mut publisher = fixture.connect();
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "build"}));
        let mut watch = subscribe(&fixture, vec!["build"], vec![], vec![]);
        assert_eq!(read(&mut watch)["op"], "snapshot-end");
        let presence = publish(&mut publisher, "build", LiveBuildFactValue::WorkerPresence {
            endpoint_id: "worker".into(),
            generation: 7,
        });
        assert_eq!(read(&mut watch)["fact_id"], presence);
        let terminal = publish(&mut publisher, "build", LiveBuildFactValue::TerminalOutcome {
            job_id: "job".into(),
            attempt_id: None,
            outcome: super::super::LiveTerminalOutcome::WorkerLost,
            terminal_phase: Some("coordination".into()),
        });
        assert_eq!(read(&mut watch)["fact_id"], terminal);
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "retract", "fact_id": presence}));
        assert_eq!(read(&mut watch)["fact_id"], presence);
        let mut current = subscribe(&fixture, vec!["build"], vec![], vec![]);
        assert_eq!(read(&mut current)["fact_id"], terminal);
        assert_eq!(read(&mut current)["op"], "snapshot-end");
        drop(publisher);
        assert_eq!(read(&mut watch)["fact_id"], terminal);
        assert_eq!(read(&mut current)["fact_id"], terminal);
    }

    #[test]
    fn duplicate_owner_cannot_steal_active_facts() {
        let fixture = Fixture::new();
        let mut original = fixture.connect();
        send(&mut original, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "same"}));
        let mut watch = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut watch)["op"], "snapshot-end");
        let first = publish(&mut original, "same", goal("first"));
        assert_eq!(read(&mut watch)["fact_id"], first);
        let mut duplicate = fixture.connect();
        send(&mut duplicate, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "same"}));
        assert_eq!(duplicate.read(&mut [0; 1]).unwrap(), 0);
        let second = publish(&mut original, "same", goal("second"));
        assert_eq!(read(&mut watch)["fact_id"], second);
        drop(original);
        let retractions = [read(&mut watch), read(&mut watch)];
        assert!(retractions.iter().all(|event| event["op"] == "retract"));
        assert_eq!(
            BTreeSet::from([
                retractions[0]["fact_id"].as_str().unwrap(),
                retractions[1]["fact_id"].as_str().unwrap()
            ]),
            BTreeSet::from([first.as_str(), second.as_str()])
        );
    }

    #[test]
    fn graceful_shutdown_retracts_and_closes_socket() {
        let fixture = Fixture::new();
        let mut publisher = fixture.connect();
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "build"}));
        let mut watch = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut watch)["op"], "snapshot-end");
        let fact_id = publish(&mut publisher, "build", goal("job"));
        assert_eq!(read(&mut watch)["fact_id"], fact_id);
        fixture.stop.store(true, Ordering::Release);
        let retract = read(&mut watch);
        assert_eq!(retract["op"], "retract");
        assert_eq!(retract["fact_id"], fact_id);
        let mut line = String::new();
        assert_eq!(watch.read_line(&mut line).unwrap(), 0);
    }

    #[test]
    fn shutdown_accepts_only_owned_terminal_readiness_exit() {
        let state = Arc::new(Mutex::new(State::default()));
        let (mut socket, reply) = UnixStream::pair().unwrap();
        let mut reply = BufReader::new(reply);
        let mut owner = None;
        let component = ReadinessComponent {
            id: "shutdown-service".to_owned(),
            kind: "service".to_owned(),
            depends_on: Vec::new(),
            user_states: Vec::new(),
            restart_policy: Some("never".to_owned()),
            stage_requirements: None,
        };
        let observation = |states: &[&str], exit: Option<&str>| ReadinessObservation {
            id: component.id.clone(),
            generation: 1,
            states: states.iter().map(|state| (*state).to_owned()).collect(),
            exit: exit.map(str::to_owned),
            request_acknowledged: false,
            proof_completion: None,
        };
        let request = |component: ReadinessComponent, observation: ReadinessObservation| ReadinessRequest::Publish {
            schema: READINESS_SCHEMA.to_owned(),
            component,
            observation,
        };
        handle_readiness_request(
            &mut socket,
            &state,
            7,
            &mut owner,
            request(component.clone(), observation(&["started"], None)),
        )
        .unwrap();
        assert_eq!(read(&mut reply)["op"], "ack");
        assert_eq!(owner.as_deref(), Some(component.id.as_str()));
        lock_state(&state).shutting_down = true;

        let denied_ready = handle_readiness_request(
            &mut socket,
            &state,
            7,
            &mut owner,
            request(component.clone(), observation(&["started", "ready"], None)),
        );
        assert_eq!(denied_ready.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        let mut foreign_owner = None;
        let foreign_component = ReadinessComponent {
            id: "foreign-service".to_owned(),
            ..component.clone()
        };
        let denied_foreign = handle_readiness_request(
            &mut socket,
            &state,
            8,
            &mut foreign_owner,
            request(foreign_component, ReadinessObservation {
                id: "foreign-service".to_owned(),
                ..observation(&["failed"], Some("abnormal"))
            }),
        );
        assert_eq!(denied_foreign.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        let denied_changed = handle_readiness_request(
            &mut socket,
            &state,
            7,
            &mut owner,
            request(
                ReadinessComponent {
                    kind: "socket".to_owned(),
                    ..component.clone()
                },
                observation(&["failed"], Some("abnormal")),
            ),
        );
        assert_eq!(denied_changed.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        let denied_restart = handle_readiness_request(
            &mut socket,
            &state,
            7,
            &mut owner,
            request(component.clone(), ReadinessObservation {
                generation: 2,
                ..observation(&["started"], None)
            }),
        );
        assert_eq!(denied_restart.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        {
            let guard = lock_state(&state);
            assert_eq!(guard.readiness_records.len(), 1);
            let prior = guard.readiness_records.get(&component.id).unwrap().observation.as_ref().unwrap();
            assert_eq!(prior.states, ["started"]);
            assert_eq!(prior.generation, 1);
            assert_eq!(prior.exit, None);
        }

        handle_readiness_request(
            &mut socket,
            &state,
            7,
            &mut owner,
            request(component.clone(), observation(&["failed"], Some("abnormal"))),
        )
        .unwrap();
        let ack = read(&mut reply);
        assert_eq!(ack["op"], "ack");
        assert_eq!(ack["id"], component.id);
        assert_eq!(ack["evidence_eligible"], false);
        let guard = lock_state(&state);
        let exited = guard.readiness_records.get(&component.id).unwrap().observation.as_ref().unwrap();
        assert_eq!(exited.states, ["failed"]);
        assert_eq!(exited.exit.as_deref(), Some("abnormal"));
    }
    #[test]
    fn disconnected_subscriber_does_not_remove_owner_facts() {
        let fixture = Fixture::new();
        let mut publisher = fixture.connect();
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "build"}));
        let mut killed = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut killed)["op"], "snapshot-end");
        let first = publish(&mut publisher, "build", goal("first"));
        assert_eq!(read(&mut killed)["fact_id"], first);
        drop(killed);
        let second = publish(&mut publisher, "build", goal("second"));
        let mut replacement = subscribe(&fixture, vec!["build"], vec![], vec![]);
        let observed = [read(&mut replacement), read(&mut replacement)];
        assert_eq!(
            BTreeSet::from([
                observed[0]["fact_id"].as_str().unwrap(),
                observed[1]["fact_id"].as_str().unwrap()
            ]),
            BTreeSet::from([first.as_str(), second.as_str()])
        );
        assert_eq!(read(&mut replacement)["op"], "snapshot-end");
    }

    #[test]
    fn stale_socket_restart_has_empty_state_and_preserves_live_socket() {
        let mut fixture = Fixture::new();
        let path = fixture.path.clone();
        let mut publisher = fixture.connect();
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "build"}));
        let mut watch = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut watch)["op"], "snapshot-end");
        let id = publish(&mut publisher, "build", goal("job"));
        assert_eq!(read(&mut watch)["fact_id"], id);
        fixture.shutdown();
        drop(publisher);
        drop(watch);
        // A dead listener leaves its AF_UNIX pathname behind like a killed daemon.
        let stale = UnixListener::bind(&path).unwrap();
        drop(stale);
        let mut restarted = Fixture::at_path(path);
        let mut current = subscribe(&restarted, vec![], vec![], vec![]);
        assert_eq!(read(&mut current)["op"], "snapshot-end");
        assert_eq!(bind_socket(&restarted.path).err().unwrap().kind(), io::ErrorKind::AddrInUse);
        restarted.shutdown();
        fs::write(&restarted.path, b"do not remove").unwrap();
        assert_eq!(bind_socket(&restarted.path).err().unwrap().kind(), io::ErrorKind::AddrInUse);
        assert_eq!(fs::read(&restarted.path).unwrap(), b"do not remove");
        fs::remove_file(&restarted.path).unwrap();
    }

    #[test]
    fn local_dispatch_without_remote_attempt_is_accepted_and_owner_retracted() {
        use crunch_pipeline::WorkerLiveGoal;
        use crunch_pipeline::WorkerLiveSnapshot;

        let fixture = Fixture::new();
        let mut publisher = fixture.connect();
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "local-build"}));
        let mut subscriber = subscribe(&fixture, vec!["local-build"], vec!["root.drv"], vec!["goal"]);
        assert_eq!(read(&mut subscriber)["op"], "snapshot-end");
        let snapshot = super::super::normalize_local_worker_live_facts("local-build", &WorkerLiveSnapshot {
            goals: vec![WorkerLiveGoal {
                drv_key: "root.drv".into(),
                state: crunch_build::GoalState::Building,
            }],
        })
        .unwrap();
        let (id, fact) = snapshot
            .facts()
            .iter()
            .find(|(_, fact)| {
                matches!(&fact.value, LiveBuildFactValue::Goal {
                    phase: LiveGoalPhase::Dispatched,
                    ..
                })
            })
            .unwrap();
        send(
            &mut publisher,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": id, "fact": fact}),
        );
        assert_eq!(read(&mut subscriber)["fact_id"], id.as_str());
        drop(publisher);
        assert_eq!(read(&mut subscriber)["op"], "retract");

        let mut invalid = fixture.connect();
        send(&mut invalid, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "remote-build"}));
        let invalid_goal = LiveBuildFactValue::Goal {
            job_id: "root.drv".into(),
            phase: LiveGoalPhase::Dispatched,
            worker_endpoint_id: Some("remote-worker".into()),
            attempt_id: None,
            fence_generation: None,
        };
        let invalid_id = fact_identity("remote-build", &invalid_goal).unwrap();
        send(
            &mut invalid,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": invalid_id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": invalid_goal}}),
        );
        assert_eq!(invalid.read(&mut [0; 1]).unwrap(), 0, "remote dispatch requires attempt and fence");
        let mut wrong_owner = fixture.connect();
        send(
            &mut wrong_owner,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "different-build"}),
        );
        let wrong_id = fact_identity("different-build", &fact.value).unwrap();
        send(
            &mut wrong_owner,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": wrong_id, "fact": fact}),
        );
        assert_eq!(wrong_owner.read(&mut [0; 1]).unwrap(), 0, "local endpoint belongs to its original owner");
    }

    #[test]
    fn slow_subscriber_is_dropped_without_blocking_publisher() {
        let fixture = Fixture::new();
        let mut publisher = fixture.connect();
        send(&mut publisher, json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "open", "owner_run_id": "build"}));
        let mut slow = fixture.connect();
        let receive_bytes: libc::c_int = 4_096;
        let result = unsafe {
            libc::setsockopt(
                slow.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_RCVBUF,
                &receive_bytes as *const _ as *const libc::c_void,
                std::mem::size_of_val(&receive_bytes) as libc::socklen_t,
            )
        };
        assert_eq!(result, 0);
        send(
            &mut slow,
            json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "subscribe", "filter": {"owner_run_ids": ["build"], "job_ids": [], "kinds": []}}),
        );
        let mut slow = BufReader::new(slow);
        assert_eq!(read(&mut slow)["op"], "snapshot-start");
        assert_eq!(read(&mut slow)["op"], "snapshot-end");
        let discovered = LiveBuildFactValue::Goal {
            job_id: "job".into(),
            phase: LiveGoalPhase::Discovered,
            worker_endpoint_id: None,
            attempt_id: Some("attempt".into()),
            fence_generation: Some(1),
        };
        let dispatched = LiveBuildFactValue::Goal {
            job_id: "job".into(),
            phase: LiveGoalPhase::Dispatched,
            worker_endpoint_id: Some("worker".into()),
            attempt_id: Some("attempt".into()),
            fence_generation: Some(1),
        };
        let id = fact_identity("build", &discovered).unwrap();
        assert_eq!(id, fact_identity("build", &dispatched).unwrap());
        let frame = |value| {
            format!(
                "{}\n",
                json!({"schema": LIVE_PROTOCOL_SCHEMA, "op": "publish", "fact_id": id, "fact": {"schema": LIVE_BUILD_FACT_SCHEMA, "value": value}})
            )
        };
        let first = frame(discovered);
        let second = frame(dispatched);
        publisher.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
        for index in 0..10_000 {
            publisher
                .write_all(if index % 2 == 0 {
                    first.as_bytes()
                } else {
                    second.as_bytes()
                })
                .unwrap();
        }
        publisher.shutdown(Shutdown::Write).unwrap();
        publisher.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
        assert_eq!(publisher.read(&mut [0; 1]).unwrap(), 0);
        let mut events = Vec::new();
        slow.read_to_end(&mut events).unwrap(); // EOF proves the bounded subscriber queue dropped it.
        assert!(events.windows(b"\"op\":\"publish\"".len()).any(|window| window == b"\"op\":\"publish\""));
        let mut fresh = subscribe(&fixture, vec![], vec![], vec![]);
        assert_eq!(read(&mut fresh)["op"], "snapshot-end");
    }
}
