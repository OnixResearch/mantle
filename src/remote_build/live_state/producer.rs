//! Best-effort, process-external publication of building-plane facts.
//!
//! The caller supplies normalized snapshots. Only the background worker touches
//! the socket; publication failures never change coordinator or store state.

use std::io::Write;
use std::io::{self};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TrySendError;
use std::sync::mpsc::{self};
use std::thread;
use std::time::Duration;

use serde::Serialize;

use super::LIVE_BUILD_FACT_SCHEMA;
use super::LiveBuildEmitAction;
use super::LiveBuildFact;
use super::LiveBuildFactError;
use super::LiveBuildSnapshot;
use super::MAX_LIVE_FACT_BYTES;
use super::MAX_LIVE_FACTS;
use super::MAX_LIVE_ID_BYTES;
use super::admit_fact_size;
use super::admit_identity;
use super::daemon::LIVE_PROTOCOL_SCHEMA;
use super::fact_identity;
use super::plan_remote_live_fact_changes;
use super::plan_remote_live_owner_stop;

const QUEUED_UPDATES: usize = 8;
const MAX_FRAME_BYTES: usize = MAX_LIVE_FACT_BYTES + MAX_LIVE_ID_BYTES + 512;
const WRITE_TIMEOUT: Duration = Duration::from_millis(250);

const CONNECTING: u8 = 0;
const CONNECTED: u8 = 1;
const ENDPOINT_MISSING: u8 = 2;
const ENDPOINT_FAILED: u8 = 3;
const QUEUE_FULL: u8 = 4;
const WORKER_FAILED: u8 = 5;
const STOPPED: u8 = 6;

/// Connection health is advisory: it is not a delivery acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LivePublisherStatus {
    Connecting,
    Connected,
    Degraded(LivePublisherDegradation),
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LivePublisherDegradation {
    EndpointMissing,
    EndpointFailed,
    QueueFull,
    WorkerFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LivePublisherObservation {
    Accepted,
    Unchanged,
    Degraded(LivePublisherDegradation),
    Stopped,
}

#[derive(Clone)]
enum OwnedAction {
    Publish { fact_id: String, fact: LiveBuildFact },
    Retract { fact_id: String },
}

enum Message {
    Update(Vec<OwnedAction>),
    Stop(Vec<OwnedAction>),
}

/// A bounded, nonblocking building-plane sink. There is no socket connection,
/// socket write, or wait for the background worker on the calling thread.
/// Dropping it disconnects the owner, so the daemon retracts any surviving facts.
pub struct BestEffortLivePublisher {
    current: Mutex<LiveBuildSnapshot>,
    sender: Option<SyncSender<Message>>,
    status: Arc<AtomicU8>,
}

impl BestEffortLivePublisher {
    pub fn new(socket_path: impl Into<PathBuf>, initial: LiveBuildSnapshot) -> Self {
        let (sender, receiver) = mpsc::sync_channel(QUEUED_UPDATES);
        let status = Arc::new(AtomicU8::new(CONNECTING));
        let worker_status = Arc::clone(&status);
        let worker_initial = initial.clone();
        let socket_path = socket_path.into();
        let worker = thread::Builder::new()
            .name("mantle-live-publisher".to_owned())
            .spawn(move || publish_loop(socket_path, worker_initial, receiver, worker_status));
        let sender = match worker {
            Ok(_worker) => Some(sender), // Detached: never joined on the build thread.
            Err(_) => {
                status.store(WORKER_FAILED, Ordering::Release);
                None
            }
        };
        Self {
            current: Mutex::new(initial),
            sender,
            status,
        }
    }

    /// Start an owner with no initial facts; subsequent `enqueue` calls supply
    /// building-plane facts without copying a snapshot at each event.
    pub fn new_empty(
        socket_path: impl Into<PathBuf>,
        owner_run_id: impl Into<String>,
    ) -> Result<Self, LiveBuildFactError> {
        let snapshot = LiveBuildSnapshot::empty(owner_run_id.into())?;
        Ok(Self::new(socket_path, snapshot))
    }

    /// Queues one *atomic* diff (retractions first). A full queue degrades the
    /// connection rather than dropping a partial diff or delaying the build.
    /// A mismatched owner is rejected without changing the accepted snapshot.
    pub fn observe(&self, next: LiveBuildSnapshot) -> Result<LivePublisherObservation, LiveBuildFactError> {
        let mut current = self.current.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let changes = plan_remote_live_fact_changes(&current, &next)?;
        if let LivePublisherStatus::Degraded(reason) = self.status() {
            return Ok(LivePublisherObservation::Degraded(reason));
        }
        let Some(sender) = &self.sender else {
            return Ok(LivePublisherObservation::Stopped);
        };
        if changes.is_empty() {
            return Ok(LivePublisherObservation::Unchanged);
        }
        let actions = changes.into_iter().map(OwnedAction::from).collect();
        match self.try_enqueue(sender, actions) {
            Ok(()) => {
                *current = next;
                Ok(self.accepted_observation())
            }
            Err(observation) => Ok(observation),
        }
    }

    /// Enqueue one already normalized live action without copying a snapshot.
    /// Identical publications and retractions of absent facts are no-ops.
    pub fn enqueue(&self, action: LiveBuildEmitAction<'_>) -> Result<LivePublisherObservation, LiveBuildFactError> {
        let mut current = self.current.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let LivePublisherStatus::Degraded(reason) = self.status() {
            return Ok(LivePublisherObservation::Degraded(reason));
        }
        let Some(sender) = &self.sender else {
            return Ok(LivePublisherObservation::Stopped);
        };
        let (actions, change) = match action {
            LiveBuildEmitAction::Publish { fact_id, fact } => {
                admit_identity(fact_id)?;
                if fact.schema != LIVE_BUILD_FACT_SCHEMA
                    || fact_identity(current.owner_run_id(), &fact.value)? != fact_id
                {
                    return Err(LiveBuildFactError::InvalidIdentity);
                }
                admit_fact_size(current.owner_run_id(), fact_id, fact)?;
                let previous = current.facts().get(fact_id);
                if previous == Some(fact) {
                    return Ok(LivePublisherObservation::Unchanged);
                }
                if previous.is_none() && current.facts().len() >= MAX_LIVE_FACTS {
                    return Err(LiveBuildFactError::TooManyFacts);
                }
                let mut actions = Vec::with_capacity(if previous.is_some() { 2 } else { 1 });
                if previous.is_some() {
                    actions.push(OwnedAction::Retract {
                        fact_id: fact_id.to_owned(),
                    });
                }
                actions.push(OwnedAction::from(LiveBuildEmitAction::Publish { fact_id, fact }));
                (actions, OwnedAction::Publish {
                    fact_id: fact_id.to_owned(),
                    fact: fact.clone(),
                })
            }
            LiveBuildEmitAction::Retract { fact_id } => {
                if !current.facts().contains_key(fact_id) {
                    return Ok(LivePublisherObservation::Unchanged);
                }
                let owned = OwnedAction::Retract {
                    fact_id: fact_id.to_owned(),
                };
                (vec![owned.clone()], owned)
            }
        };
        match self.try_enqueue(sender, actions) {
            Ok(()) => {
                match change {
                    OwnedAction::Publish { fact_id, fact } => {
                        current.facts.insert(fact_id, fact);
                    }
                    OwnedAction::Retract { fact_id } => {
                        current.facts.remove(&fact_id);
                    }
                }
                Ok(self.accepted_observation())
            }
            Err(observation) => Ok(observation),
        }
    }

    fn try_enqueue(
        &self,
        sender: &SyncSender<Message>,
        actions: Vec<OwnedAction>,
    ) -> Result<(), LivePublisherObservation> {
        match sender.try_send(Message::Update(actions)) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                degrade(&self.status, QUEUE_FULL);
                Err(self.degraded_observation())
            }
            Err(TrySendError::Disconnected(_)) => {
                degrade(&self.status, ENDPOINT_FAILED);
                Err(self.degraded_observation())
            }
        }
    }

    fn accepted_observation(&self) -> LivePublisherObservation {
        match self.status() {
            LivePublisherStatus::Degraded(reason) => LivePublisherObservation::Degraded(reason),
            _ => LivePublisherObservation::Accepted,
        }
    }

    pub fn status(&self) -> LivePublisherStatus {
        match self.status.load(Ordering::Acquire) {
            CONNECTING => LivePublisherStatus::Connecting,
            CONNECTED => LivePublisherStatus::Connected,
            ENDPOINT_MISSING => LivePublisherStatus::Degraded(LivePublisherDegradation::EndpointMissing),
            ENDPOINT_FAILED => LivePublisherStatus::Degraded(LivePublisherDegradation::EndpointFailed),
            QUEUE_FULL => LivePublisherStatus::Degraded(LivePublisherDegradation::QueueFull),
            WORKER_FAILED => LivePublisherStatus::Degraded(LivePublisherDegradation::WorkerFailed),
            _ => LivePublisherStatus::Stopped,
        }
    }

    /// Signals owner shutdown without waiting for I/O. A queued stop retracts
    /// explicitly; if the queue is full, disconnect alone retracts at the daemon.
    pub fn finish(&mut self) {
        let Some(sender) = self.sender.take() else { return };
        if !matches!(self.status(), LivePublisherStatus::Connecting | LivePublisherStatus::Connected) {
            return;
        }
        let current = self.current.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let actions = plan_remote_live_owner_stop(&current).into_iter().map(OwnedAction::from).collect();
        if let Err(TrySendError::Full(_)) = sender.try_send(Message::Stop(actions)) {
            degrade(&self.status, QUEUE_FULL);
        }
        // The sender is dropped here; after the queued work the worker drops
        // its UnixStream even if it did not receive a Stop message.
    }

    fn degraded_observation(&self) -> LivePublisherObservation {
        match self.status() {
            LivePublisherStatus::Degraded(reason) => LivePublisherObservation::Degraded(reason),
            _ => LivePublisherObservation::Stopped,
        }
    }
}

impl Drop for BestEffortLivePublisher {
    fn drop(&mut self) {
        self.finish();
    }
}

impl<'a> From<LiveBuildEmitAction<'a>> for OwnedAction {
    fn from(action: LiveBuildEmitAction<'a>) -> Self {
        match action {
            LiveBuildEmitAction::Publish { fact_id, fact } => Self::Publish {
                fact_id: fact_id.to_owned(),
                fact: fact.clone(),
            },
            LiveBuildEmitAction::Retract { fact_id } => Self::Retract {
                fact_id: fact_id.to_owned(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
enum WireRequest<'a> {
    Open {
        schema: &'static str,
        owner_run_id: &'a str,
    },
    Publish {
        schema: &'static str,
        fact_id: &'a str,
        fact: &'a LiveBuildFact,
    },
    Retract {
        schema: &'static str,
        fact_id: &'a str,
    },
}

fn publish_loop(socket_path: PathBuf, initial: LiveBuildSnapshot, receiver: Receiver<Message>, status: Arc<AtomicU8>) {
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(error) => {
            degrade(
                &status,
                if error.kind() == io::ErrorKind::NotFound {
                    ENDPOINT_MISSING
                } else {
                    ENDPOINT_FAILED
                },
            );
            return;
        }
    };
    if stream.set_write_timeout(Some(WRITE_TIMEOUT)).is_err() {
        degrade(&status, ENDPOINT_FAILED);
        return;
    }
    let mut frame = Vec::with_capacity(MAX_FRAME_BYTES);
    if send_if_active(&mut stream, &mut frame, &status, &WireRequest::Open {
        schema: LIVE_PROTOCOL_SCHEMA,
        owner_run_id: initial.owner_run_id(),
    })
    .is_err()
    {
        degrade(&status, ENDPOINT_FAILED);
        return;
    }
    for (fact_id, fact) in initial.facts() {
        if send_if_active(&mut stream, &mut frame, &status, &WireRequest::Publish {
            schema: LIVE_PROTOCOL_SCHEMA,
            fact_id,
            fact,
        })
        .is_err()
        {
            degrade(&status, ENDPOINT_FAILED);
            return;
        }
    }
    let _ = status.compare_exchange(CONNECTING, CONNECTED, Ordering::AcqRel, Ordering::Acquire);
    while let Ok(message) = receiver.recv() {
        let (actions, stop) = match message {
            Message::Update(actions) => (actions, false),
            Message::Stop(actions) => (actions, true),
        };
        for action in &actions {
            let request = match action {
                OwnedAction::Publish { fact_id, fact } => WireRequest::Publish {
                    schema: LIVE_PROTOCOL_SCHEMA,
                    fact_id,
                    fact,
                },
                OwnedAction::Retract { fact_id } => WireRequest::Retract {
                    schema: LIVE_PROTOCOL_SCHEMA,
                    fact_id,
                },
            };
            if send_if_active(&mut stream, &mut frame, &status, &request).is_err() {
                degrade(&status, ENDPOINT_FAILED);
                return;
            }
        }
        if stop {
            break;
        }
    }
    let _ = status.compare_exchange(CONNECTED, STOPPED, Ordering::AcqRel, Ordering::Acquire);
}

fn send_if_active(
    stream: &mut UnixStream,
    frame: &mut Vec<u8>,
    status: &AtomicU8,
    request: &WireRequest<'_>,
) -> io::Result<()> {
    if matches!(status.load(Ordering::Acquire), ENDPOINT_MISSING | ENDPOINT_FAILED | QUEUE_FULL | WORKER_FAILED) {
        return Err(io::Error::new(io::ErrorKind::BrokenPipe, "live publisher degraded"));
    }
    frame.clear();
    serde_json::to_writer(&mut *frame, request)?;
    if frame.len() >= MAX_FRAME_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "live publisher frame exceeds bound"));
    }
    frame.push(b'\n');
    stream.write_all(frame)
}

fn degrade(status: &AtomicU8, reason: u8) {
    let _ = status.fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
        matches!(previous, CONNECTING | CONNECTED).then_some(reason)
    });
}
