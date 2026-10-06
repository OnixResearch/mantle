//! Ephemeral local coordination state. This service neither opens a store nor authors evidence.
//! A reset invalidates the client's entire previous view: after a daemon restart
//! old connections are gone, so retractions from the old generation cannot be delivered.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crunch_live_state_core::Fact;
use crunch_live_state_core::FactKind;
use crunch_live_state_core::Filter;
use crunch_live_state_core::LiveSet;
use crunch_live_state_core::MAX_FACT_BYTES;
use crunch_live_state_core::MAX_FACTS;
use crunch_live_state_core::MAX_FILTER_BYTES;
use crunch_live_state_core::MAX_FRAME_BYTES;
use crunch_live_state_core::MAX_PENDING_EVENTS;
use crunch_live_state_core::MAX_SUBSCRIBERS;
use crunch_live_state_core::fact_id;
use crunch_service_readiness_core::RestartPolicy;
use crunch_service_readiness_core::ServiceAssertion;
use crunch_service_readiness_core::ServiceDeclaration;
use crunch_service_readiness_core::ServiceEvent;
use crunch_service_readiness_core::ServiceGraph;
use crunch_service_readiness_core::ServiceState;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::sync::watch;

const MAX_CONNECTIONS: usize = MAX_SUBSCRIBERS.saturating_mul(2);
/// Aggregate budget for captured atomic snapshots (eight maximum-sized full sets).
pub const MAX_ACTIVE_SNAPSHOT_BYTES: usize = 8_usize.saturating_mul(MAX_FACTS).saturating_mul(MAX_FACT_BYTES);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
type Bytes = Arc<[u8]>;

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Subscribe { filter: Filter },
    Publish { fact: Fact },
    Retract { id: String },
}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum PublisherRequest<'a> {
    Publish { fact: &'a Fact },
    Retract { id: &'a str },
}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Response<'a> {
    Reset { generation: &'a str },
    Snapshot { fact: &'a Fact },
    SnapshotEnd,
    Publish { fact: &'a Fact },
    Retract { id: &'a str },
    Ack,
    Error { message: &'a str },
}

// The snapshot job captures an immutable point-in-time view but materializes
// wire frames one at a time; the queue still admits only MAX_PENDING_EVENTS
// later changes, and all simultaneous captures share one aggregate byte cap.
struct SnapshotLease {
    facts: Vec<Fact>,
    bytes: usize,
    budget: Arc<AtomicUsize>,
}

impl Drop for SnapshotLease {
    fn drop(&mut self) {
        self.budget.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

enum Outbound {
    One(Bytes),
    Snapshot(SnapshotLease),
}

struct Subscriber {
    filter: Filter,
    send: mpsc::Sender<Outbound>,
    stop: watch::Sender<bool>,
}

#[derive(Default)]
struct State {
    facts: LiveSet,
    subscribers: HashMap<u64, Subscriber>,
    connections: HashSet<u64>,
    owner_sessions: HashMap<String, u64>,
    ids: HashMap<String, u64>,
    next_id: u64,
    snapshot_budget: Arc<AtomicUsize>,
    ready_signal: Option<watch::Sender<bool>>,
}

impl State {
    fn broadcast(&mut self, fact: &Fact, frame: Bytes) {
        self.subscribers.retain(|id, subscriber| {
            if subscriber.filter.matches(fact) && subscriber.send.try_send(Outbound::One(frame.clone())).is_err() {
                subscriber.stop.send_replace(true);
                eprintln!("mantle-coordination: subscriber {id} dropped (pending events exceeded bound)");
                false
            } else {
                true
            }
        });
    }

    fn retract_fact(&mut self, id: &str) -> Result<Option<Fact>, String> {
        if !self.ids.contains_key(id) {
            return Ok(None);
        }
        let response = frame(&Response::Retract { id })?;
        let fact = self.facts.retract(id).ok_or_else(|| "known fact disappeared before retraction".to_owned())?;
        self.ids.remove(id);
        self.broadcast(&fact, response);
        Ok(Some(fact))
    }

    fn owner_readiness(&self, owner: &str, exclude_id: Option<&str>) -> Result<Vec<ServiceAssertion>, String> {
        let filter = Filter {
            owner: Some(owner.to_owned()),
            kind: Some(FactKind::ServiceReadiness),
            subject_prefix: None,
        };
        self.facts
            .snapshot(&filter)
            .filter(|fact| exclude_id != Some(fact.id.as_str()))
            .map(parse_readiness)
            .collect()
    }

    fn reconcile_readiness(&mut self, owner: &str) -> Result<(), String> {
        for _ in 0..MAX_FACTS {
            let active = self.owner_readiness(owner, None)?;
            // r[impl mantle.service_readiness.declared_dependencies]
            let invalid = ServiceGraph::invalid_ready_subjects(&active).map_err(|error| error.to_string())?;
            if invalid.is_empty() {
                return Ok(());
            }
            for subject in invalid {
                let id = fact_id(owner, FactKind::ServiceReadiness, &subject);
                if self.retract_fact(&id)?.is_none() {
                    return Err("invalid readiness subject was not active".to_owned());
                }
            }
        }
        Err("readiness reconciliation exceeded active fact bound".to_owned())
    }

    fn retire_service(&mut self, owner: &str, service_id: &str) -> Result<(), String> {
        let active = self.owner_readiness(owner, None)?;
        for assertion in active {
            if assertion.service_id == service_id
                && !matches!(assertion.state, ServiceState::Complete | ServiceState::Failed)
            {
                let id = fact_id(owner, FactKind::ServiceReadiness, &assertion.subject());
                self.retract_fact(&id)?;
            }
        }
        self.reconcile_readiness(owner)
    }

    fn publish_owned(&mut self, id: u64, owner: &mut Option<String>, fact: Fact) -> Result<(), String> {
        fact.validate().map_err(|error| error.to_string())?;
        if owner.as_ref().is_some_and(|bound| bound != &fact.owner) {
            return Err("publisher cannot change owner".to_owned());
        }
        if self.owner_sessions.get(&fact.owner).is_some_and(|session| *session != id) {
            return Err("owner already has a publisher".to_owned());
        }
        if self.ids.get(&fact.id).is_some_and(|session| *session != id) {
            return Err("fact belongs to another publisher".to_owned());
        }
        let readiness = if fact.kind == FactKind::ServiceReadiness {
            // r[impl mantle.service_readiness.readiness_vocabulary]
            // r[impl mantle.service_readiness.declared_dependencies]
            let incoming = parse_readiness(&fact)?;
            let existing = self.owner_readiness(&fact.owner, Some(&fact.id))?;
            ServiceGraph::admit_active(&existing, &incoming).map_err(|error| error.to_string())?;
            Some(incoming)
        } else {
            None
        };
        let is_changed = self.facts.publish(fact.clone()).map_err(|error| error.to_string())?;
        if owner.is_none() {
            self.owner_sessions.insert(fact.owner.clone(), id);
            *owner = Some(fact.owner.clone());
        }
        self.ids.insert(fact.id.clone(), id);
        if is_changed {
            self.broadcast(&fact, frame(&Response::Publish { fact: &fact })?);
            if let Some(assertion) = readiness
                && matches!(assertion.state, ServiceState::Complete | ServiceState::Failed)
            {
                self.retire_service(&fact.owner, &assertion.service_id)?;
            }
        }
        Ok(())
    }

    fn retract_owned(&mut self, session: u64, id: &str) -> Result<(), String> {
        if id.len() > MAX_FRAME_BYTES {
            return Err("fact id exceeds bound".to_owned());
        }
        if self.ids.get(id).is_some_and(|owner| *owner != session) {
            return Err("fact belongs to another publisher".to_owned());
        }
        if self.ids.contains_key(id)
            && let Some(fact) = self.retract_fact(id)?
            && fact.kind == FactKind::ServiceReadiness
        {
            self.reconcile_readiness(&fact.owner)?;
        }
        Ok(())
    }

    fn disconnect(&mut self, id: u64, owner: Option<&str>) {
        self.connections.remove(&id);
        self.subscribers.remove(&id);
        if let Some(owner) = owner {
            self.owner_sessions.remove(owner);
            // r[impl mantle.coordination_service.retraction_on_owner_stop]
            for fact in self.facts.retract_owner(owner) {
                self.ids.remove(&fact.id);
                match frame(&Response::Retract { id: &fact.id }) {
                    Ok(response) => self.broadcast(&fact, response),
                    Err(error) => eprintln!("mantle-coordination: cannot encode owner retraction: {error}"),
                }
            }
        }
    }
}

fn parse_readiness(fact: &Fact) -> Result<ServiceAssertion, String> {
    // r[impl mantle.service_readiness.readiness_vocabulary]
    let assertion: ServiceAssertion = serde_json::from_str(&fact.state).map_err(|error| error.to_string())?;
    assertion.validate().map_err(|error| error.to_string())?;
    if fact.subject != assertion.subject() {
        return Err("readiness subject does not match typed assertion".to_owned());
    }
    Ok(assertion)
}

fn frame(value: &Response<'_>) -> Result<Bytes, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FRAME_BYTES {
        if !matches!(value, Response::Error { .. }) {
            return Err("coordination response exceeds frame bound".to_owned());
        }
        bytes = serde_json::to_vec(&Response::Error {
            message: "error message exceeds frame bound",
        })
        .map_err(|error| error.to_string())?;
        bytes.push(b'\n');
    }
    Ok(Bytes::from(bytes))
}

fn queue(send: &mpsc::Sender<Outbound>, stop: &watch::Sender<bool>, response: Response<'_>) {
    match frame(&response) {
        Ok(bytes) => {
            if send.try_send(Outbound::One(bytes)).is_err() {
                stop.send_replace(true);
            }
        }
        Err(error) => {
            eprintln!("mantle-coordination: cannot encode response: {error}");
            stop.send_replace(true);
        }
    }
}

struct ClientSession<'a> {
    id: u64,
    owner: &'a mut Option<String>,
    send: &'a mpsc::Sender<Outbound>,
    stop: &'a watch::Sender<bool>,
}

// r[impl mantle.coordination_service.live_state_subscription]
fn apply(state: &mut State, session: &mut ClientSession<'_>, request: Request) -> Result<(), String> {
    debug_assert!(session.id > 0);
    debug_assert!(state.connections.contains(&session.id));
    match request {
        Request::Subscribe { filter } => {
            filter.validate().map_err(|error| error.to_string())?;
            let filter_bytes = serde_json::to_vec(&filter).map_err(|error| error.to_string())?;
            if filter_bytes.len() > MAX_FILTER_BYTES {
                return Err("filter exceeds byte bound".to_owned());
            }
            if state.subscribers.contains_key(&session.id) {
                return Err("connection already subscribed".to_owned());
            }
            if state.subscribers.len() >= MAX_SUBSCRIBERS {
                return Err("subscriber limit reached".to_owned());
            }
            // Capture and register under one lock, then write the snapshot
            // outside the lock. Deltas queue behind snapshot_end in the
            // connection's writer, without delaying a publisher.
            let facts: Vec<Fact> = state.facts.snapshot(&filter).cloned().collect();
            debug_assert!(facts.len() <= MAX_FACTS);
            let bytes = facts.len().checked_mul(MAX_FACT_BYTES).ok_or("snapshot byte count overflow")?;
            debug_assert!(bytes <= MAX_ACTIVE_SNAPSHOT_BYTES);
            state
                .snapshot_budget
                .try_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                    used.checked_add(bytes).filter(|total| *total <= MAX_ACTIVE_SNAPSHOT_BYTES)
                })
                .map_err(|_| "active snapshot byte budget reached".to_owned())?;
            let snapshot = SnapshotLease {
                facts,
                bytes,
                budget: state.snapshot_budget.clone(),
            };
            if session.send.try_send(Outbound::Snapshot(snapshot)).is_err() {
                session.stop.send_replace(true);
                return Err("subscriber queue full".to_owned());
            }
            state.subscribers.insert(session.id, Subscriber {
                filter,
                send: session.send.clone(),
                stop: session.stop.clone(),
            });
            if let Some(signal) = &state.ready_signal {
                signal.send_replace(true);
            }
        }
        Request::Publish { fact } => {
            state.publish_owned(session.id, session.owner, fact)?;
            queue(session.send, session.stop, Response::Ack);
        }
        Request::Retract { id: fact_id } => {
            state.retract_owned(session.id, &fact_id)?;
            queue(session.send, session.stop, Response::Ack);
        }
    }
    Ok(())
}

async fn read_line<R: AsyncRead + Unpin>(reader: &mut BufReader<R>) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    for _ in 0..=MAX_FRAME_BYTES {
        debug_assert!(line.len() <= MAX_FRAME_BYTES);
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "unterminated request"))
            };
        }
        let count = match available.iter().position(|byte| *byte == b'\n') {
            Some(pos) => pos
                .checked_add(1)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "NDJSON frame position overflow"))?,
            None => available.len(),
        };
        debug_assert!(count <= available.len());
        if line.len().checked_add(count).is_none_or(|bytes| bytes > MAX_FRAME_BYTES) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "NDJSON frame exceeds byte bound"));
        }
        line.extend_from_slice(&available[..count]);
        reader.consume(count);
        if line.last() == Some(&b'\n') {
            return Ok(Some(line));
        }
    }
    Err(io::Error::new(io::ErrorKind::InvalidData, "NDJSON frame exceeded read bound"))
}

async fn write_one(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    bytes: &[u8],
    cancelled: &mut watch::Receiver<bool>,
) -> io::Result<bool> {
    tokio::select! {
        _ = cancelled.changed() => Ok(false),
        result = tokio::time::timeout(WRITE_TIMEOUT, writer.write_all(bytes)) => {
            result.map_err(io::Error::other)?.map(|()| true)
        },
    }
}

async fn write_frames(
    mut writer: tokio::net::unix::OwnedWriteHalf,
    mut receiver: mpsc::Receiver<Outbound>,
    mut cancelled: watch::Receiver<bool>,
    stop: watch::Sender<bool>,
) {
    loop {
        let event = tokio::select! {
            _ = cancelled.changed() => break,
            event = receiver.recv() => event,
        };
        let Some(event) = event else { break };
        match event {
            Outbound::One(bytes) => {
                if !matches!(write_one(&mut writer, &bytes, &mut cancelled).await, Ok(true)) {
                    stop.send_replace(true);
                    return;
                }
            }
            Outbound::Snapshot(snapshot) => {
                debug_assert!(snapshot.facts.len() <= MAX_FACTS);
                debug_assert!(snapshot.bytes <= MAX_ACTIVE_SNAPSHOT_BYTES);
                for fact in &snapshot.facts {
                    let bytes = match frame(&Response::Snapshot { fact }) {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            eprintln!("mantle-coordination: cannot encode snapshot: {error}");
                            stop.send_replace(true);
                            return;
                        }
                    };
                    if !matches!(write_one(&mut writer, &bytes, &mut cancelled).await, Ok(true)) {
                        stop.send_replace(true);
                        return;
                    }
                }
                let end = match frame(&Response::SnapshotEnd) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        eprintln!("mantle-coordination: cannot encode snapshot end: {error}");
                        stop.send_replace(true);
                        return;
                    }
                };
                if !matches!(write_one(&mut writer, &end, &mut cancelled).await, Ok(true)) {
                    stop.send_replace(true);
                    return;
                }
            }
        }
    }
    stop.send_replace(true);
}

// A poisoned shared set cannot safely keep publishing purportedly live facts.
// Terminate the daemon so clients observe EOF and retract the entire generation.
fn lock_state(state: &Mutex<State>) -> std::sync::MutexGuard<'_, State> {
    match state.lock() {
        Ok(guard) => guard,
        Err(_) => {
            eprintln!("mantle-coordination: shared state poisoned; stopping daemon");
            std::process::abort();
        }
    }
}

async fn connection(stream: UnixStream, id: u64, shared: Arc<Mutex<State>>, generation: Arc<str>) {
    debug_assert!(id > 0);
    debug_assert!(!generation.is_empty());
    let (read_half, write_half) = stream.into_split();
    let (send, receiver) = mpsc::channel(MAX_PENDING_EVENTS);
    let (stop, mut cancelled) = watch::channel(false);
    queue(&send, &stop, Response::Reset {
        generation: &generation,
    });
    let writer = tokio::spawn(write_frames(write_half, receiver, cancelled.clone(), stop.clone()));
    let mut reader = BufReader::new(read_half);
    let mut owner = None;
    loop {
        let input = tokio::select! {
            _ = cancelled.changed() => break,
            input = read_line(&mut reader) => input,
        };
        let bytes = match input {
            Ok(Some(bytes)) => bytes,
            Ok(None) => break,
            Err(error) => {
                queue(&send, &stop, Response::Error {
                    message: &error.to_string(),
                });
                break;
            }
        };
        match serde_json::from_slice::<Request>(&bytes) {
            Ok(request) => {
                let result = {
                    let mut guard = lock_state(&shared);
                    let mut session = ClientSession {
                        id,
                        owner: &mut owner,
                        send: &send,
                        stop: &stop,
                    };
                    apply(&mut guard, &mut session, request)
                };
                if let Err(message) = result {
                    queue(&send, &stop, Response::Error { message: &message });
                }
            }
            Err(error) => queue(&send, &stop, Response::Error {
                message: &error.to_string(),
            }),
        }
        if *cancelled.borrow() {
            break;
        }
    }
    lock_state(&shared).disconnect(id, owner.as_deref());
    stop.send_replace(true);
    drop(send);
    if let Err(error) = writer.await {
        eprintln!("mantle-coordination: connection writer failed: {error}");
    }
}

struct OwnedSocket {
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Drop for OwnedSocket {
    fn drop(&mut self) {
        let Ok(metadata) = fs::symlink_metadata(&self.path) else {
            return;
        };
        if !metadata.file_type().is_socket() {
            return;
        }
        let is_owned = metadata.dev() == self.device && metadata.ino() == self.inode;
        if is_owned && let Err(error) = fs::remove_file(&self.path) {
            eprintln!("mantle-coordination: could not remove owned socket: {error}");
        }
    }
}

fn bind_socket(path: &Path) -> io::Result<(OwnedSocket, UnixListener)> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_socket() {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, "socket path is occupied"));
            }
            match std::os::unix::net::UnixStream::connect(path) {
                Ok(_) => return Err(io::Error::new(io::ErrorKind::AddrInUse, "another daemon owns socket")),
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
                    let current = fs::symlink_metadata(path)?;
                    if current.dev() != metadata.dev() || current.ino() != metadata.ino() {
                        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "socket changed during stale probe"));
                    }
                    fs::remove_file(path)?;
                }
                Err(error) => return Err(error),
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let listener = UnixListener::bind(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "socket changed during bind"));
    }
    let owned = OwnedSocket {
        path: path.to_owned(),
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    let secured = fs::symlink_metadata(path)?;
    let is_owned_socket =
        secured.file_type().is_socket() && secured.dev() == owned.device && secured.ino() == owned.inode;
    let is_private = secured.permissions().mode() & 0o777 == 0o600;
    if !is_owned_socket || !is_private {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "socket changed during permission setup"));
    }
    debug_assert!(secured.file_type().is_socket());
    debug_assert_eq!(secured.permissions().mode() & 0o777, 0o600);
    Ok((owned, listener))
}

fn daemon_assertion(
    graph: &mut ServiceGraph,
    event: ServiceEvent,
    state: ServiceState,
) -> io::Result<ServiceAssertion> {
    // r[impl mantle.service_readiness.restart_policy_matrix]
    graph
        .apply("coordination", event)
        .map_err(|error| io::Error::other(error.to_string()))?
        .assertions
        .into_iter()
        .find(|assertion| assertion.state == state)
        .ok_or_else(|| io::Error::other("daemon readiness transition omitted requested assertion"))
}

fn daemon_fact(owner: &str, assertion: &ServiceAssertion) -> io::Result<Fact> {
    let state = serde_json::to_string(assertion).map_err(io::Error::other)?;
    Fact::new(owner, FactKind::ServiceReadiness, &assertion.subject(), &state)
        .map_err(|error| io::Error::other(error.to_string()))
}

// Linux daemon and blocking-client shell boundary. Realtime retains an epoch
// generation; monotonic samples enforce absolute deadlines across retries.
fn clock_time(clock_id: libc::clockid_t) -> io::Result<Duration> {
    let mut raw: libc::timespec = unsafe { std::mem::zeroed() };
    if unsafe { libc::clock_gettime(clock_id, &raw mut raw) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let seconds = u64::try_from(raw.tv_sec)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "clock reported negative seconds"))?;
    let nanos = u32::try_from(raw.tv_nsec)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "clock reported negative nanoseconds"))?;
    if nanos >= 1_000_000_000 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "clock nanoseconds exceed one second"));
    }
    Ok(Duration::new(seconds, nanos))
}

fn daemon_graph() -> io::Result<ServiceGraph> {
    ServiceGraph::new(vec![ServiceDeclaration {
        service_id: "coordination".to_owned(),
        custom_states: Vec::new(),
        dependencies: Vec::new(),
        restart_policy: RestartPolicy::Never,
    }])
    .map_err(|error| io::Error::other(error.to_string()))
}

fn daemon_terminal_fact(graph: &mut ServiceGraph, owner: &str, is_ready: bool) -> io::Result<Fact> {
    let (event, terminal) = if is_ready {
        (ServiceEvent::Complete, ServiceState::Complete)
    } else {
        (ServiceEvent::Failed, ServiceState::Failed)
    };
    let assertion = daemon_assertion(graph, event, terminal)?;
    daemon_fact(owner, &assertion)
}

enum DaemonEvent {
    Client(UnixStream),
    FirstSubscription,
    Shutdown,
}

/// Serve ephemeral, connection-owned facts. The daemon's own typed `started`
/// fact is established before accepting clients; a distinct `ready` fact
/// appears only after an actual accepted subscription. A new process starts
/// with a new generation; reconnecting subscribers retract prior IDs on reset.
pub async fn serve(socket: &Path) -> io::Result<()> {
    let (owned_socket, listener) = bind_socket(socket)?;
    let startup = clock_time(libc::CLOCK_REALTIME)?;
    let generation: Arc<str> = format!("{:x}-{:x}", startup.as_nanos(), std::process::id()).into();
    debug_assert!(!generation.is_empty());
    let owner_name = format!("mantle-coordination-{generation}");
    let mut owner = None;
    let mut graph = daemon_graph()?;
    // r[impl mantle.service_readiness.readiness_vocabulary]
    let started = daemon_assertion(&mut graph, ServiceEvent::Started, ServiceState::Started)?;
    let state = Arc::new(Mutex::new(State::default()));
    let (ready_sender, mut ready_receiver) = watch::channel(false);
    {
        let mut guard = lock_state(&state);
        guard.ready_signal = Some(ready_sender);
        guard.publish_owned(0, &mut owner, daemon_fact(&owner_name, &started)?).map_err(io::Error::other)?;
    }
    debug_assert_eq!(owner.as_deref(), Some(owner_name.as_str()));
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut is_ready = false;
    loop {
        let next = tokio::select! {
            result = listener.accept() => DaemonEvent::Client(result?.0),
            _ = ready_receiver.changed(), if !is_ready => DaemonEvent::FirstSubscription,
            _ = tokio::signal::ctrl_c() => DaemonEvent::Shutdown,
            _ = terminate.recv() => DaemonEvent::Shutdown,
        };
        match next {
            DaemonEvent::FirstSubscription => {
                if *ready_receiver.borrow() {
                    let assertion = daemon_assertion(&mut graph, ServiceEvent::Ready, ServiceState::Ready)?;
                    lock_state(&state)
                        .publish_owned(0, &mut owner, daemon_fact(&owner_name, &assertion)?)
                        .map_err(io::Error::other)?;
                    is_ready = true;
                }
            }
            DaemonEvent::Client(stream) => {
                let id = {
                    let mut guard = lock_state(&state);
                    if guard.connections.len() >= MAX_CONNECTIONS {
                        continue;
                    }
                    guard.next_id =
                        guard.next_id.checked_add(1).ok_or_else(|| io::Error::other("connection counter exhausted"))?;
                    let id = guard.next_id;
                    guard.connections.insert(id);
                    id
                };
                tokio::spawn(connection(stream, id, state.clone(), generation.clone()));
            }
            DaemonEvent::Shutdown => break,
        }
    }
    lock_state(&state)
        .publish_owned(0, &mut owner, daemon_terminal_fact(&mut graph, &owner_name, is_ready)?)
        .map_err(io::Error::other)?;
    drop(listener);
    // Give already connected subscribers a bounded chance to consume the
    // terminal assertion and retractions before the process closes sockets.
    tokio::time::sleep(Duration::from_millis(20)).await;
    drop(owned_socket);
    Ok(())
}

/// Observations emitted by [`SubscriptionClient`]. A daemon restart closes
/// existing sockets; the client materializes one retraction for each old fact
/// before forwarding the next generation's reset and snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionEvent {
    Reset { generation: String },
    Snapshot(Fact),
    SnapshotEnd,
    Publish(Fact),
    Retract { id: String },
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Incoming {
    Reset { generation: String },
    Snapshot { fact: Fact },
    SnapshotEnd,
    Publish { fact: Fact },
    Retract { id: String },
    Ack,
    Error { message: String },
}

fn publisher_ack(incoming: Incoming) -> io::Result<()> {
    match incoming {
        Incoming::Ack => Ok(()),
        Incoming::Error { message } => Err(io::Error::new(io::ErrorKind::InvalidData, message)),
        Incoming::Reset { .. }
        | Incoming::Snapshot { .. }
        | Incoming::SnapshotEnd
        | Incoming::Publish { .. }
        | Incoming::Retract { .. } => {
            Err(io::Error::new(io::ErrorKind::InvalidData, "expected publisher acknowledgment"))
        }
    }
}

const CLIENT_TIMEOUT: Duration = Duration::from_secs(2);

fn client_timeout(error: tokio::time::error::Elapsed) -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, error)
}

fn publisher_frame(request: &PublisherRequest<'_>) -> io::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(request).map_err(io::Error::other)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "publisher frame exceeds byte bound"));
    }
    Ok(bytes)
}

async fn incoming_frame(reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>) -> io::Result<Incoming> {
    let line = tokio::time::timeout(CLIENT_TIMEOUT, read_line(reader))
        .await
        .map_err(client_timeout)?
        .and_then(|line| line.ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof)))?;
    serde_json::from_slice(&line).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// A single publisher connection owns its facts until explicit retraction or
/// disconnection. Commands wait for daemon acknowledgment with bounded I/O.
/// On any write/ack failure this session closes its write side; do not reuse it.
pub struct PublisherSession {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
    generation: String,
    failed: bool,
}

impl PublisherSession {
    pub async fn connect(socket: &Path) -> io::Result<Self> {
        let stream =
            tokio::time::timeout(CLIENT_TIMEOUT, UnixStream::connect(socket)).await.map_err(client_timeout)??;
        let (reader, writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let generation = match incoming_frame(&mut reader).await? {
            Incoming::Reset { generation } if !generation.is_empty() => generation,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected daemon reset")),
        };
        Ok(Self {
            reader,
            writer,
            generation,
            failed: false,
        })
    }

    pub fn generation(&self) -> &str {
        &self.generation
    }

    async fn command(&mut self, request: PublisherRequest<'_>) -> io::Result<()> {
        if self.failed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "publisher session failed"));
        }
        let bytes = publisher_frame(&request)?;
        let result = async {
            tokio::time::timeout(CLIENT_TIMEOUT, self.writer.write_all(&bytes))
                .await
                .map_err(client_timeout)??;
            publisher_ack(incoming_frame(&mut self.reader).await?)
        }
        .await;
        if result.is_err() {
            self.failed = true;
            match tokio::time::timeout(CLIENT_TIMEOUT, self.writer.shutdown()).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => eprintln!("mantle-coordination: publisher shutdown failed: {error}"),
                Err(_) => eprintln!("mantle-coordination: publisher shutdown timed out"),
            }
        }
        result
    }

    pub async fn publish(&mut self, fact: &Fact) -> io::Result<()> {
        fact.validate().map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
        self.command(PublisherRequest::Publish { fact }).await
    }

    pub async fn retract(&mut self, id: &str) -> io::Result<()> {
        if id.len() > MAX_FRAME_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "fact id exceeds byte bound"));
        }
        self.command(PublisherRequest::Retract { id }).await
    }
}

fn blocking_deadline() -> io::Result<Duration> {
    clock_time(libc::CLOCK_MONOTONIC)?
        .checked_add(CLIENT_TIMEOUT)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "client deadline overflow"))
}

fn remaining(deadline: Duration, now: Duration) -> io::Result<Duration> {
    let time = deadline.saturating_sub(now);
    if time.is_zero() {
        return Err(io::Error::new(io::ErrorKind::TimedOut, "coordination operation timed out"));
    }
    Ok(time)
}

fn unix_address(socket: &Path) -> io::Result<(libc::sockaddr_un, libc::socklen_t)> {
    let path_bytes = socket.as_os_str().as_bytes();
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if path_bytes.is_empty() || path_bytes.contains(&0) || path_bytes.len() >= address.sun_path.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid Unix socket path"));
    }
    debug_assert!(!path_bytes.is_empty());
    debug_assert!(path_bytes.len() < address.sun_path.len());
    address.sun_family = libc::sa_family_t::try_from(libc::AF_UNIX)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid Unix socket family"))?;
    for (slot, byte) in address.sun_path.iter_mut().zip(path_bytes) {
        *slot = *byte as libc::c_char;
    }
    let length_bytes = std::mem::offset_of!(libc::sockaddr_un, sun_path)
        .checked_add(path_bytes.len())
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "socket address length overflow"))?;
    let length = libc::socklen_t::try_from(length_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "socket address too long"))?;
    Ok((address, length))
}

fn blocking_connect(socket: &Path) -> io::Result<StdUnixStream> {
    let (address, length) = unix_address(socket)?;
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: socket returned a newly owned descriptor, now guarded through every error path.
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    debug_assert!(owned.as_raw_fd() >= 0);
    let result = unsafe { libc::connect(owned.as_raw_fd(), (&raw const address).cast::<libc::sockaddr>(), length) };
    if result < 0 {
        let error = io::Error::last_os_error();
        if !matches!(error.raw_os_error(), Some(libc::EINPROGRESS | libc::EINTR | libc::EALREADY)) {
            return Err(error);
        }
        let deadline = blocking_deadline()?;
        loop {
            let timeout_ms = i32::try_from(remaining(deadline, clock_time(libc::CLOCK_MONOTONIC)?)?.as_millis())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "client timeout exceeds poll range"))?;
            let mut probe = libc::pollfd {
                fd: owned.as_raw_fd(),
                events: libc::POLLOUT,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&raw mut probe, 1, timeout_ms) };
            if ready == 0 {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Unix socket connect timed out"));
            }
            if ready > 0 {
                break;
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        let mut socket_error = 0_i32;
        let mut socket_error_len = std::mem::size_of::<i32>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                owned.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&raw mut socket_error).cast(),
                &raw mut socket_error_len,
            )
        } < 0
        {
            return Err(io::Error::last_os_error());
        }
        if socket_error != 0 {
            return Err(io::Error::from_raw_os_error(socket_error));
        }
        debug_assert_eq!(socket_error, 0);
    }
    let stream: StdUnixStream = owned.into();
    stream.set_nonblocking(false)?;
    Ok(stream)
}

fn blocking_incoming(reader: &mut io::BufReader<StdUnixStream>) -> io::Result<Incoming> {
    let deadline = blocking_deadline()?;
    let mut line = Vec::new();
    for _ in 0..=MAX_FRAME_BYTES {
        debug_assert!(line.len() <= MAX_FRAME_BYTES);
        reader.get_mut().set_read_timeout(Some(remaining(deadline, clock_time(libc::CLOCK_MONOTONIC)?)?))?;
        let available = io::BufRead::fill_buf(reader).map_err(|error| {
            if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) {
                io::Error::new(io::ErrorKind::TimedOut, "coordination response timed out")
            } else {
                error
            }
        })?;
        if available.is_empty() {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        let count = match available.iter().position(|byte| *byte == b'\n') {
            Some(pos) => pos
                .checked_add(1)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "response frame position overflow"))?,
            None => available.len(),
        };
        debug_assert!(count <= available.len());
        if line.len().checked_add(count).is_none_or(|bytes| bytes > MAX_FRAME_BYTES) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "response frame exceeds byte bound"));
        }
        line.extend_from_slice(&available[..count]);
        io::BufRead::consume(reader, count);
        if line.last() == Some(&b'\n') {
            return serde_json::from_slice(&line).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
        }
    }
    Err(io::Error::new(io::ErrorKind::InvalidData, "response frame exceeded read bound"))
}

/// Synchronous publisher for callers without a Tokio runtime. One session
/// keeps its Unix socket open across all changes; dropping it retracts facts.
pub struct BlockingPublisherSession {
    reader: io::BufReader<StdUnixStream>,
    writer: StdUnixStream,
    generation: String,
    failed: bool,
}

impl BlockingPublisherSession {
    pub fn connect(socket: &Path) -> io::Result<Self> {
        let writer = blocking_connect(socket)?;
        let mut reader = io::BufReader::new(writer.try_clone()?);
        let generation = match blocking_incoming(&mut reader)? {
            Incoming::Reset { generation } if !generation.is_empty() => generation,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected daemon reset")),
        };
        Ok(Self {
            reader,
            writer,
            generation,
            failed: false,
        })
    }

    pub fn generation(&self) -> &str {
        &self.generation
    }

    fn command(&mut self, request: PublisherRequest<'_>) -> io::Result<()> {
        if self.failed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "publisher session failed"));
        }
        let bytes = publisher_frame(&request)?;
        let result = (|| {
            let deadline = blocking_deadline()?;
            let mut offset_bytes = 0;
            while offset_bytes < bytes.len() {
                self.writer.set_write_timeout(Some(remaining(deadline, clock_time(libc::CLOCK_MONOTONIC)?)?))?;
                match io::Write::write(&mut self.writer, &bytes[offset_bytes..]) {
                    Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero)),
                    Ok(count) => {
                        offset_bytes = offset_bytes.checked_add(count).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "publisher write offset overflow")
                        })?;
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => return Err(error),
                }
            }
            publisher_ack(blocking_incoming(&mut self.reader)?)
        })();
        if result.is_err() {
            self.failed = true;
            if let Err(error) = self.writer.shutdown(Shutdown::Both) {
                eprintln!("mantle-coordination: blocking publisher shutdown failed: {error}");
            }
        }
        result
    }

    pub fn publish(&mut self, fact: &Fact) -> io::Result<()> {
        fact.validate().map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
        self.command(PublisherRequest::Publish { fact })
    }

    pub fn retract(&mut self, id: &str) -> io::Result<()> {
        if id.len() > MAX_FRAME_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "fact id exceeds byte bound"));
        }
        self.command(PublisherRequest::Retract { id })
    }
}

/// Upper bound for one-shot service shutdown while draining queued ACKs.
pub const MAX_END_FLUSH: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BestEffortError {
    Unavailable,
    Full,
    InvalidFact(String),
}

impl std::fmt::Display for BestEffortError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("coordination publisher unavailable"),
            Self::Full => formatter.write_str("coordination publisher queue full"),
            Self::InvalidFact(message) => write!(formatter, "invalid live fact: {message}"),
        }
    }
}

impl std::error::Error for BestEffortError {}

enum BestEffortAction {
    Publish(Fact),
    Retract(String),
    Finish(std::sync::mpsc::SyncSender<()>),
}

/// Nonblocking producer for service critical paths. One background thread
/// owns exactly one connection, with MAX_PENDING_EVENTS queued commands.
/// Any dropped command degrades observation and ends the session, retracting
/// its facts rather than leaving falsely live assertions behind.
pub struct BestEffortPublisher {
    sender: Option<std::sync::mpsc::SyncSender<BestEffortAction>>,
    healthy: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

impl BestEffortPublisher {
    pub fn new(socket: &Path) -> io::Result<Self> {
        let path = socket.to_owned();
        let healthy = Arc::new(AtomicBool::new(true));
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = std::sync::mpsc::sync_channel(MAX_PENDING_EVENTS);
        let worker_healthy = healthy.clone();
        let worker_stop = stop.clone();
        let _worker =
            std::thread::Builder::new().name("mantle-coordination-publisher".to_owned()).spawn(move || {
                let Ok(mut publisher) = BlockingPublisherSession::connect(&path) else {
                    worker_healthy.store(false, Ordering::Release);
                    return;
                };
                while let Ok(action) = receiver.recv() {
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    let result = match action {
                        BestEffortAction::Publish(fact) => publisher.publish(&fact),
                        BestEffortAction::Retract(id) => publisher.retract(&id),
                        BestEffortAction::Finish(done) => {
                            drop(publisher);
                            worker_healthy.store(false, Ordering::Release);
                            if done.send(()).is_err() {
                                eprintln!(
                                    "mantle-coordination: completion receiver closed before flush acknowledgment"
                                );
                            }
                            return;
                        }
                    };
                    if result.is_err() {
                        break;
                    }
                }
                worker_healthy.store(false, Ordering::Release);
            })?;
        Ok(Self {
            sender: Some(sender),
            healthy,
            stop,
        })
    }

    pub fn is_degraded(&self) -> bool {
        !self.healthy.load(Ordering::Acquire)
    }

    fn enqueue(&self, action: BestEffortAction) -> Result<(), BestEffortError> {
        if self.is_degraded() {
            return Err(BestEffortError::Unavailable);
        }
        let sender = self.sender.as_ref().ok_or(BestEffortError::Unavailable)?;
        match sender.try_send(action) {
            Ok(()) => Ok(()),
            Err(std::sync::mpsc::TrySendError::Full(_)) => {
                self.healthy.store(false, Ordering::Release);
                self.stop.store(true, Ordering::Release);
                Err(BestEffortError::Full)
            }
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                self.healthy.store(false, Ordering::Release);
                Err(BestEffortError::Unavailable)
            }
        }
    }

    pub fn try_publish(&self, fact: Fact) -> Result<(), BestEffortError> {
        if self.is_degraded() {
            return Err(BestEffortError::Unavailable);
        }
        fact.validate().map_err(|error| BestEffortError::InvalidFact(error.to_string()))?;
        self.enqueue(BestEffortAction::Publish(fact))
    }

    pub fn try_retract(&self, id: &str) -> Result<(), BestEffortError> {
        if id.len() > MAX_FRAME_BYTES {
            return Err(BestEffortError::InvalidFact("fact id exceeds byte bound".to_owned()));
        }
        self.enqueue(BestEffortAction::Retract(id.to_owned()))
    }

    /// Try to drain ACKs before a one-shot process exits; never wait longer
    /// than 100ms. Returns false if the daemon is missing, stalled or full.
    pub fn finish(self, deadline: Duration) -> bool {
        let (done, completed) = std::sync::mpsc::sync_channel(1);
        if self.enqueue(BestEffortAction::Finish(done)).is_err() {
            return false;
        }
        completed.recv_timeout(deadline.min(MAX_END_FLUSH)).is_ok()
    }
}

impl Drop for BestEffortPublisher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.sender.take();
    }
}

/// A reconnecting subscriber retains only its current ephemeral fact set.
/// Call `disconnected` on EOF, then feed the next socket's frames to `receive`.
/// A reset also retracts all retained facts, including when an EOF was missed.
#[derive(Default)]
pub struct SubscriptionClient {
    facts: std::collections::BTreeMap<String, Fact>,
    generation: Option<String>,
}

impl SubscriptionClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generation(&self) -> Option<&str> {
        self.generation.as_deref()
    }

    pub fn current(&self) -> impl Iterator<Item = &Fact> {
        self.facts.values()
    }

    pub fn disconnected(&mut self) -> Vec<SubscriptionEvent> {
        self.generation = None;
        std::mem::take(&mut self.facts).into_keys().map(|id| SubscriptionEvent::Retract { id }).collect()
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Result<Vec<SubscriptionEvent>, String> {
        if bytes.len() > MAX_FRAME_BYTES {
            return Err("response frame exceeds byte bound".to_owned());
        }
        let incoming: Incoming = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        match incoming {
            Incoming::Reset { generation } => {
                let mut events = self.disconnected();
                self.generation = Some(generation.clone());
                events.push(SubscriptionEvent::Reset { generation });
                Ok(events)
            }
            Incoming::Snapshot { fact } => {
                fact.validate().map_err(|error| error.to_string())?;
                self.facts.insert(fact.id.clone(), fact.clone());
                Ok(vec![SubscriptionEvent::Snapshot(fact)])
            }
            Incoming::SnapshotEnd => Ok(vec![SubscriptionEvent::SnapshotEnd]),
            Incoming::Publish { fact } => {
                fact.validate().map_err(|error| error.to_string())?;
                self.facts.insert(fact.id.clone(), fact.clone());
                Ok(vec![SubscriptionEvent::Publish(fact)])
            }
            Incoming::Retract { id } => {
                self.facts.remove(&id);
                Ok(vec![SubscriptionEvent::Retract { id }])
            }
            Incoming::Ack => Ok(Vec::new()),
            Incoming::Error { message } => Err(message),
        }
    }
}
