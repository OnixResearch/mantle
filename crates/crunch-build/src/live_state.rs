//! Best-effort, session-owned live build observations. Never used for build decisions.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use crunch_live_state_core::Fact;
use crunch_live_state_core::FactKind;
use crunch_live_state_core::LiveSet;
use crunch_live_state_core::MAX_FRAME_BYTES;
use crunch_live_state_core::MAX_PENDING_EVENTS;
use crunch_live_state_core::fact_id;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::goal::GoalState;

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
// One process-owned diagnostic writer and one pending notice. A full stderr
// can stall this worker, but never creates another thread or queues unbounded
// notices from subsequent builds.
const MAX_PENDING_DIAGNOSTICS: usize = 1;
#[cfg(test)]
static DIAGNOSTIC_WORKERS_CREATED: AtomicU64 = AtomicU64::new(0);
static DIAGNOSTICS_DROPPED: AtomicU64 = AtomicU64::new(0);
static DIAGNOSTICS: LazyLock<Option<SyncSender<&'static str>>> = LazyLock::new(|| {
    let (sender, receiver) = std::sync::mpsc::sync_channel(MAX_PENDING_DIAGNOSTICS);
    std::thread::Builder::new()
        .spawn(move || {
            while let Ok(reason) = receiver.recv() {
                eprintln!(
                    "{}",
                    serde_json::json!({"schema":"mantle-live-observation-v1","status":"degraded","reason":reason})
                );
            }
        })
        .ok()
        .map(|_| {
            #[cfg(test)]
            DIAGNOSTIC_WORKERS_CREATED.fetch_add(1, Ordering::Relaxed);
            sender
        })
});
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);
const END_FLUSH_BOUND: Duration = Duration::from_millis(100);

fn diagnostic_sender() -> Option<&'static SyncSender<&'static str>> {
    DIAGNOSTICS.as_ref()
}

/// Process-local, saturating count of notices lost when stderr cannot keep up.
/// This is observation state, never a build result or evidence receipt.
pub fn live_observation_diagnostic_drops() -> u64 {
    DIAGNOSTICS_DROPPED.load(Ordering::Relaxed)
}

fn report_degraded(reported: &AtomicBool, reason: &'static str) {
    if !reported.swap(true, Ordering::Relaxed)
        && diagnostic_sender().is_none_or(|sender| sender.try_send(reason).is_err())
    {
        let _ =
            DIAGNOSTICS_DROPPED.try_update(Ordering::Relaxed, Ordering::Relaxed, |count| Some(count.saturating_add(1)));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Discovered,
    Ready,
    Building,
    Done,
    Failed,
}

fn phase(state: &GoalState) -> Phase {
    match state {
        GoalState::AwaitingDerivation | GoalState::Pending | GoalState::Waiting { .. } => Phase::Discovered,
        GoalState::Ready => Phase::Ready,
        GoalState::Building => Phase::Building,
        GoalState::Done => Phase::Done,
        GoalState::Failed => Phase::Failed,
    }
}

/// The only state kept by this adapter is observation state; it cannot influence scheduling.
pub(crate) struct LivePublisher {
    owner: String,
    sender: Option<mpsc::Sender<Vec<u8>>>,
    facts: LiveSet,
    observed: BTreeMap<String, Phase>,
    reported: Arc<AtomicBool>,
    writer: Option<tokio::task::JoinHandle<()>>,
}

impl LivePublisher {
    pub(crate) fn from_env() -> Option<Self> {
        let socket = std::env::var_os("MANTLE_COORDINATION_SOCKET")?;
        if socket.is_empty() {
            report_degraded(&AtomicBool::new(false), "empty_endpoint");
            return None;
        }
        Some(Self::connect_socket(socket))
    }

    fn connect_socket(socket: std::ffi::OsString) -> Self {
        let owner = format!("build-{}-{}", std::process::id(), NEXT_SESSION.fetch_add(1, Ordering::Relaxed));
        let (sender, receiver) = mpsc::channel(MAX_PENDING_EVENTS);
        let reported = Arc::new(AtomicBool::new(false));
        let writer = tokio::spawn(write_session(socket, receiver, Arc::clone(&reported)));
        let mut publisher = Self {
            owner,
            sender: Some(sender),
            facts: LiveSet::new(),
            observed: BTreeMap::new(),
            reported,
            writer: Some(writer),
        };
        publisher.publish(FactKind::Worker, "scheduler", "running");
        publisher
    }

    pub(crate) fn degrade(&mut self, reason: &'static str) {
        // Dropping the sender alone would keep a stalled writer (and all its
        // already-published facts) alive while it drained old queued frames.
        // Abort the owner connection immediately so the daemon retracts them.
        if let Some(writer) = self.writer.take() {
            writer.abort();
        }
        self.sender.take();
        report_degraded(&self.reported, reason);
    }

    fn send(&mut self, value: serde_json::Value) {
        if self.sender.is_none() {
            return;
        }
        let Ok(mut frame) = serde_json::to_vec(&value) else {
            self.degrade("serialization_failure");
            return;
        };
        if frame.len().saturating_add(1) > MAX_FRAME_BYTES {
            self.degrade("frame_limit");
            return;
        }
        frame.push(b'\n');
        if self.sender.as_ref().is_some_and(|sender| sender.try_send(frame).is_err()) {
            self.degrade("publisher_queue_full_or_closed");
        }
    }

    fn publish(&mut self, kind: FactKind, subject: &str, state: &str) {
        if self.sender.is_none() {
            return;
        }
        let fact = match Fact::new(&self.owner, kind, subject, state) {
            Ok(fact) => fact,
            Err(_) => {
                self.degrade("invalid_fact");
                return;
            }
        };
        match self.facts.publish(fact.clone()) {
            Ok(true) => self.send(serde_json::json!({"op": "publish", "fact": fact})),
            Ok(false) => (),
            Err(_) => self.degrade("fact_rejected"),
        }
    }

    fn retract(&mut self, kind: FactKind, subject: &str) {
        if self.sender.is_none() {
            return;
        }
        let id = fact_id(&self.owner, kind, subject);
        if self.facts.retract(&id).is_some() {
            self.send(serde_json::json!({"op": "retract", "id": id}));
        }
    }

    /// Normalize one real state transition rather than scanning the registry
    /// during dispatch; repeated discovered/waiting states are no-ops.
    pub(crate) fn observe_transition(&mut self, key: &str, state: &GoalState) {
        if self.sender.is_none() {
            return;
        }
        let next = phase(state);
        let previous = self.observed.get(key).copied();
        if previous == Some(next) {
            return;
        }
        if previous == Some(Phase::Building) {
            self.retract(FactKind::Reservation, key);
        }
        if previous.is_some() {
            self.retract(FactKind::Goal, key);
        }
        match next {
            Phase::Discovered => self.publish(FactKind::Goal, key, "discovered"),
            Phase::Ready => self.publish(FactKind::Goal, key, "ready"),
            Phase::Building => {
                self.publish(FactKind::Goal, key, "building");
                self.publish(FactKind::Reservation, key, "dispatched");
            }
            Phase::Done => self.publish(FactKind::Outcome, key, "success"),
            Phase::Failed => self.publish(FactKind::Outcome, key, "failure"),
        }
        if self.sender.is_some() {
            self.observed.insert(key.to_owned(), next);
        }
    }

    /// Active facts are withdrawn explicitly. Terminal outcomes remain valid
    /// until the owner connection closes, which retracts all remaining facts.
    pub(crate) async fn finish(mut self) {
        for (key, phase) in std::mem::take(&mut self.observed) {
            if phase == Phase::Building {
                self.retract(FactKind::Reservation, &key);
            }
            if !matches!(phase, Phase::Done | Phase::Failed) {
                self.retract(FactKind::Goal, &key);
            }
        }
        self.retract(FactKind::Worker, "scheduler");
        self.sender.take();
        let timed_out = if let Some(writer) = self.writer.as_mut() {
            timeout(END_FLUSH_BOUND, writer).await.is_err()
        } else {
            false
        };
        if timed_out {
            if let Some(writer) = self.writer.as_mut() {
                writer.abort();
            }
            report_degraded(&self.reported, "end_flush_timeout");
        }
        self.writer.take();
    }
}

impl Drop for LivePublisher {
    fn drop(&mut self) {
        // A cancelled worker must not leave a detached writer presenting
        // in-flight goals as still live after its scheduler disappears.
        if let Some(writer) = self.writer.take() {
            writer.abort();
        }
    }
}

async fn read_reply(reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>) -> std::io::Result<serde_json::Value> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "coordination endpoint closed"));
        }
        let count = available.iter().position(|byte| *byte == b'\n').map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(count) > MAX_FRAME_BYTES {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "coordination reply too large"));
        }
        line.extend_from_slice(&available[..count]);
        reader.consume(count);
        if line.last() == Some(&b'\n') {
            return serde_json::from_slice(&line)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error));
        }
    }
}

async fn write_session(socket: std::ffi::OsString, mut receiver: mpsc::Receiver<Vec<u8>>, reported: Arc<AtomicBool>) {
    let stream = timeout(WRITE_TIMEOUT, UnixStream::connect(std::path::Path::new(&socket))).await;
    let stream = match stream {
        Ok(Ok(stream)) => stream,
        Ok(Err(_)) => {
            report_degraded(&reported, "endpoint_unavailable");
            return;
        }
        Err(_) => {
            report_degraded(&reported, "connect_timeout");
            return;
        }
    };
    // Each publish/retract waits for its daemon ack, off the scheduler path.
    // This bounds unacknowledged requests to one and keeps daemon queues safe.
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    while let Some(frame) = receiver.recv().await {
        match timeout(WRITE_TIMEOUT, write_half.write_all(&frame)).await {
            Ok(Ok(())) => (),
            Ok(Err(_)) => {
                report_degraded(&reported, "socket_write_failure");
                return;
            }
            Err(_) => {
                report_degraded(&reported, "socket_write_timeout");
                return;
            }
        }
        // The first response is a connection reset/generation, then one ack
        // per request. Consume both without retaining an unbounded read buffer.
        loop {
            let reply = timeout(WRITE_TIMEOUT, read_reply(&mut reader)).await;
            match reply {
                Ok(Ok(value)) if value["op"] == "reset" => continue,
                Ok(Ok(value)) if value["op"] == "ack" => break,
                Ok(Ok(_)) => report_degraded(&reported, "endpoint_rejected"),
                Ok(Err(_)) => report_degraded(&reported, "endpoint_disconnected"),
                Err(_) => report_degraded(&reported, "socket_read_timeout"),
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use nix_compat::derivation::Derivation;
    use nix_compat::derivation::Output;
    use nix_compat::store_path::StorePath;
    use tokio::io::AsyncReadExt;

    use super::*;
    use crate::goal::Goal;
    use crate::goal::GoalRegistry;

    fn fixture() -> (LivePublisher, mpsc::Receiver<Vec<u8>>, GoalRegistry, String) {
        let path = StorePath::from_name_and_digest_fixed("root.drv", [0_u8; 20]).unwrap();
        let key = path.to_absolute_path();
        let derivation = Derivation {
            arguments: vec![],
            builder: "/bin/sh".into(),
            environment: BTreeMap::new(),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs: BTreeMap::from([("out".into(), Output {
                path: None,
                ca_hash: None,
            })]),
            system: "x86_64-linux".into(),
        };
        let mut registry = GoalRegistry::new();
        registry.insert(key.clone(), Goal::new(path, Arc::new(derivation))).unwrap();
        let (sender, receiver) = mpsc::channel(MAX_PENDING_EVENTS);
        let publisher = LivePublisher {
            owner: "test-worker".into(),
            sender: Some(sender),
            facts: LiveSet::new(),
            observed: BTreeMap::new(),
            reported: Arc::new(AtomicBool::new(false)),
            writer: None,
        };
        (publisher, receiver, registry, key)
    }

    fn next(receiver: &mut mpsc::Receiver<Vec<u8>>) -> serde_json::Value {
        serde_json::from_slice(&receiver.try_recv().unwrap()).unwrap()
    }

    #[tokio::test]
    async fn dispatch_then_completion_retracts_live_goal_and_reservation_but_retains_outcome() {
        let (mut publisher, mut frames, mut registry, key) = fixture();
        publisher.observe_transition(&key, &registry.get(&key).unwrap().state);
        assert_eq!(next(&mut frames)["fact"]["state"], "discovered");
        registry.get_mut(&key).unwrap().state = GoalState::Ready;
        publisher.observe_transition(&key, &registry.get(&key).unwrap().state);
        assert_eq!(next(&mut frames)["op"], "retract");
        assert_eq!(next(&mut frames)["fact"]["state"], "ready");
        registry.get_mut(&key).unwrap().state = GoalState::Building;
        publisher.observe_transition(&key, &registry.get(&key).unwrap().state);
        assert_eq!(next(&mut frames)["op"], "retract");
        assert_eq!(next(&mut frames)["fact"]["state"], "building");
        assert_eq!(next(&mut frames)["fact"]["kind"], "reservation");
        registry.get_mut(&key).unwrap().state = GoalState::Done;
        publisher.observe_transition(&key, &registry.get(&key).unwrap().state);
        assert_eq!(next(&mut frames)["id"], fact_id("test-worker", FactKind::Reservation, &key));
        assert_eq!(next(&mut frames)["id"], fact_id("test-worker", FactKind::Goal, &key));
        assert_eq!(next(&mut frames)["fact"]["state"], "success");
        assert!(frames.try_recv().is_err());
        publisher.finish().await;
        assert!(frames.try_recv().is_err(), "terminal outcome stays until session disconnect");
    }

    #[tokio::test]
    async fn dependency_failure_has_no_fake_reservation_and_disconnected_observer_is_nonfatal() {
        let (mut publisher, mut frames, mut registry, key) = fixture();
        publisher.observe_transition(&key, &registry.get(&key).unwrap().state);
        let discovered = next(&mut frames);
        assert_eq!(discovered["fact"]["kind"], "goal");
        registry.get_mut(&key).unwrap().state = GoalState::Failed;
        publisher.observe_transition(&key, &registry.get(&key).unwrap().state);
        assert_eq!(next(&mut frames)["id"], fact_id("test-worker", FactKind::Goal, &key));
        assert_eq!(next(&mut frames)["fact"]["state"], "failure");
        assert!(frames.try_recv().is_err(), "no slot was dispatched");
        drop(frames);
        publisher.finish().await;
    }

    #[tokio::test]
    async fn missing_and_nonreading_endpoints_never_hold_build_completion() {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let missing = temp.path().join("missing.sock");
        let publisher = LivePublisher::connect_socket(missing.into_os_string());
        let reported = Arc::clone(&publisher.reported);
        tokio::time::timeout(Duration::from_millis(500), publisher.finish()).await.unwrap();
        assert!(reported.load(Ordering::Relaxed), "missing endpoint must report degradation");

        let socket = temp.path().join("unresponsive.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let (connected, ready) = tokio::sync::oneshot::channel();
        let peer = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let _ = connected.send(());
            tokio::time::sleep(Duration::from_secs(2)).await;
            drop(stream);
        });
        let publisher = LivePublisher::connect_socket(socket.into_os_string());
        ready.await.unwrap();
        let reported = Arc::clone(&publisher.reported);
        tokio::time::timeout(Duration::from_millis(500), publisher.finish()).await.unwrap();
        assert!(reported.load(Ordering::Relaxed), "unresponsive endpoint must report degradation");
        peer.abort();
    }

    #[tokio::test]
    async fn degraded_publisher_closes_owner_socket_before_build_finishes() {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let socket = temp.path().join("degraded.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let (connected, ready) = tokio::sync::oneshot::channel();
        let peer = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            writer.write_all(b"{\"op\":\"reset\",\"generation\":\"test\"}\n").await.unwrap();
            let mut reader = BufReader::new(reader);
            let mut first = String::new();
            reader.read_line(&mut first).await.unwrap();
            assert_eq!(serde_json::from_str::<serde_json::Value>(&first).unwrap()["fact"]["kind"], "worker");
            writer.write_all(b"{\"op\":\"ack\"}\n").await.unwrap();
            let _ = connected.send(());
            let mut bytes = [0_u8; 256];
            loop {
                match reader.read(&mut bytes).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => (),
                }
            }
        });
        let mut publisher = LivePublisher::connect_socket(socket.into_os_string());
        ready.await.unwrap();
        publisher.publish(FactKind::Goal, "pending", "building");
        publisher.degrade("publisher_queue_full_or_closed");
        assert!(publisher.sender.is_none());
        assert!(publisher.writer.is_none());
        tokio::time::timeout(Duration::from_millis(500), peer).await.unwrap().unwrap();
        publisher.finish().await;
    }

    #[test]
    fn repeated_degraded_builds_keep_one_bounded_diagnostic_writer() {
        let dropped_before = live_observation_diagnostic_drops();
        for _ in 0..256 {
            let reported = AtomicBool::new(false);
            report_degraded(&reported, "endpoint_unavailable");
            assert!(reported.load(Ordering::Relaxed));
        }
        assert_eq!(DIAGNOSTIC_WORKERS_CREATED.load(Ordering::Relaxed), 1);
        if std::env::var_os("MANTLE_TEST_LIVE_STDERR_FULL").is_some() {
            assert!(live_observation_diagnostic_drops() > dropped_before);
        }
    }
}
