use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use crunch_coordination::BestEffortError;
use crunch_coordination::BestEffortPublisher;
use crunch_coordination::BlockingPublisherSession;
use crunch_coordination::MAX_END_FLUSH;
use crunch_coordination::PublisherSession;
use crunch_coordination::SubscriptionClient;
use crunch_coordination::SubscriptionEvent;
use crunch_live_state_core::Fact;
use crunch_live_state_core::FactKind;
use crunch_live_state_core::Filter;
use crunch_live_state_core::MAX_FILTER_BYTES;
use crunch_service_readiness_core::RestartPolicy;
use crunch_service_readiness_core::SCHEMA;
use crunch_service_readiness_core::ServiceAssertion;
use crunch_service_readiness_core::ServiceState;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::UnixStream;
use tokio::net::unix::OwnedReadHalf;
use tokio::net::unix::OwnedWriteHalf;

const WAIT: Duration = Duration::from_secs(10);

struct Daemon {
    _directory: TempDir,
    socket: PathBuf,
    child: Child,
}

impl Daemon {
    async fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("coord.sock");
        let child = launch(&socket);
        wait_for_socket(&socket).await;
        Self {
            _directory: directory,
            socket,
            child,
        }
    }

    async fn restart(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        self.child = launch(&self.socket);
        wait_for_socket(&self.socket).await;
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn launch(socket: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_mantle-coordination"))
        .args(["serve", "--socket"])
        .arg(socket)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

async fn wait_for_socket(path: &Path) {
    tokio::time::timeout(WAIT, async {
        loop {
            if path.symlink_metadata().is_ok_and(|metadata| metadata.file_type().is_socket())
                && UnixStream::connect(path).await.is_ok()
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("daemon did not bind socket");
}

struct Peer {
    read: BufReader<OwnedReadHalf>,
    write: OwnedWriteHalf,
}

impl Peer {
    async fn connect(socket: &Path) -> Self {
        Self::connect_with_reset(socket).await.0
    }

    async fn connect_with_reset(socket: &Path) -> (Self, Value) {
        let stream = UnixStream::connect(socket).await.unwrap();
        let (read, write) = stream.into_split();
        let mut peer = Self {
            read: BufReader::new(read),
            write,
        };
        let reset = peer.next().await;
        assert_eq!(reset["op"], "reset");
        (peer, reset)
    }

    async fn send(&mut self, value: Value) {
        let mut bytes = serde_json::to_vec(&value).unwrap();
        bytes.push(b'\n');
        self.write.write_all(&bytes).await.unwrap();
    }

    async fn next(&mut self) -> Value {
        let mut line = String::new();
        let size = tokio::time::timeout(WAIT, self.read.read_line(&mut line))
            .await
            .expect("response timed out")
            .unwrap();
        assert!(size > 0, "connection closed");
        serde_json::from_str(&line).unwrap()
    }

    async fn subscribe(&mut self, filter: Filter) {
        self.send(json!({ "op": "subscribe", "filter": filter })).await;
    }

    async fn publish(&mut self, fact: &Fact) {
        self.send(json!({ "op": "publish", "fact": fact })).await;
        assert_eq!(self.next().await["op"], "ack");
    }

    async fn retract(&mut self, id: &str) {
        self.send(json!({ "op": "retract", "id": id })).await;
        assert_eq!(self.next().await["op"], "ack");
    }
}

fn owned(owner: &str) -> Filter {
    Filter {
        owner: Some(owner.to_owned()),
        kind: None,
        subject_prefix: None,
    }
}

fn readiness(owner: &str, service: &str, state: ServiceState) -> Fact {
    readiness_with(owner, service, state, &[], None)
}

fn readiness_with(
    owner: &str,
    service: &str,
    state: ServiceState,
    dependencies: &[&str],
    blocked_by: Option<&str>,
) -> Fact {
    let assertion = ServiceAssertion {
        schema: SCHEMA.to_owned(),
        service_id: service.to_owned(),
        state,
        custom_states: Vec::new(),
        dependencies: dependencies.iter().map(|dependency| (*dependency).to_owned()).collect(),
        restart_policy: RestartPolicy::Never,
        blocked_by: blocked_by.map(str::to_owned),
        coordination_state: true,
    };
    Fact::new(owner, FactKind::ServiceReadiness, &assertion.subject(), &serde_json::to_string(&assertion).unwrap())
        .unwrap()
}

fn asserted_state(value: &Value) -> ServiceState {
    serde_json::from_str::<ServiceAssertion>(value["fact"]["state"].as_str().unwrap()).unwrap().state
}

#[tokio::test]
async fn snapshot_precedes_change_and_filters_multiple_roots() {
    let daemon = Daemon::start().await;
    let mut publisher = Peer::connect(&daemon.socket).await;
    let initial = Fact::new("run-a", FactKind::Goal, "root/a", "discovered").unwrap();
    let other = Fact::new("run-a", FactKind::Goal, "other/c", "discovered").unwrap();
    publisher.publish(&initial).await;
    publisher.publish(&other).await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer
        .subscribe(Filter {
            owner: Some("run-a".to_owned()),
            kind: Some(FactKind::Goal),
            subject_prefix: Some("root/".to_owned()),
        })
        .await;
    assert_eq!(observer.next().await["fact"]["id"], initial.id);
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let root_b = Fact::new("run-a", FactKind::Goal, "root/b", "dispatched").unwrap();
    publisher.publish(&root_b).await;
    assert_eq!(observer.next().await["fact"]["id"], root_b.id);
    publisher.retract(&initial.id).await;
    assert_eq!(observer.next().await["id"], initial.id);
    let terminal = Fact::new("run-a", FactKind::Outcome, "root/b", "succeeded").unwrap();
    publisher.publish(&terminal).await;
    publisher.retract(&root_b.id).await;
    assert_eq!(observer.next().await["id"], root_b.id);
    let mut current = Peer::connect(&daemon.socket).await;
    current.subscribe(owned("run-a")).await;
    let found = [current.next().await, current.next().await];
    assert!(found.iter().any(|value| value["fact"]["id"] == other.id));
    assert!(found.iter().any(|value| value["fact"]["id"] == terminal.id));
    assert_eq!(current.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn lost_worker_retracts_its_facts_not_an_unrelated_root() {
    let daemon = Daemon::start().await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("worker-session")).await;
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let mut worker = Peer::connect(&daemon.socket).await;
    let mut unrelated = Peer::connect(&daemon.socket).await;
    let presence = Fact::new("worker-session", FactKind::Worker, "worker/1", "present").unwrap();
    let in_flight = Fact::new("worker-session", FactKind::Goal, "root/a", "dispatched").unwrap();
    let retained = Fact::new("other-session", FactKind::Goal, "root/b", "building").unwrap();
    worker.publish(&presence).await;
    worker.publish(&in_flight).await;
    unrelated.publish(&retained).await;
    for _ in 0..2 {
        assert_eq!(observer.next().await["op"], "publish");
    }
    drop(worker);
    let retracts = [observer.next().await, observer.next().await];
    assert!(retracts.iter().all(|event| event["op"] == "retract"));
    assert!(retracts.iter().any(|event| event["id"] == presence.id));
    assert!(retracts.iter().any(|event| event["id"] == in_flight.id));
    let mut later = Peer::connect(&daemon.socket).await;
    later.subscribe(owned("other-session")).await;
    assert_eq!(later.next().await["fact"]["id"], retained.id);
    assert_eq!(later.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn rejects_bad_frames_oversized_filter_forged_fact_and_foreign_retract() {
    let daemon = Daemon::start().await;
    assert_eq!(daemon.socket.symlink_metadata().unwrap().permissions().mode() & 0o777, 0o600);
    let mut owner = Peer::connect(&daemon.socket).await;
    let fact = readiness("owner", "api", ServiceState::Started);
    owner.publish(&fact).await;
    let mut malicious = Peer::connect(&daemon.socket).await;
    malicious.send(json!({"op":"retract","id":fact.id})).await;
    assert_eq!(malicious.next().await["op"], "error");
    malicious.send(json!({"op":"publish","fact":fact})).await;
    assert_eq!(malicious.next().await["op"], "error");
    malicious.send(json!({"op":"publish","fact":{ "version":1, "id":"forged", "owner":"wrong", "kind":"goal", "subject":"a", "state":"b"}})).await;
    assert_eq!(malicious.next().await["op"], "error");
    malicious.send(json!({"op":"subscribe","filter":{ "owner":null,"kind":null,"subject_prefix":"x".repeat(MAX_FILTER_BYTES)}})).await;
    assert_eq!(malicious.next().await["op"], "error");
    malicious.send(json!({"op":"mutate_store","path":"/nix/store"})).await;
    assert_eq!(malicious.next().await["op"], "error");
    malicious.send(json!({"op":"subscribe","filter":owned("owner")})).await;
    assert_eq!(malicious.next().await["fact"]["id"], fact.id);
    assert_eq!(malicious.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn subscriber_disconnect_does_not_affect_publisher() {
    let daemon = Daemon::start().await;
    let mut dead = Peer::connect(&daemon.socket).await;
    dead.subscribe(owned("run")).await;
    assert_eq!(dead.next().await["op"], "snapshot_end");
    drop(dead);
    let mut publisher = Peer::connect(&daemon.socket).await;
    let fact = Fact::new("run", FactKind::Goal, "root", "building").unwrap();
    publisher.publish(&fact).await;
    let mut later = Peer::connect(&daemon.socket).await;
    later.subscribe(owned("run")).await;
    assert_eq!(later.next().await["fact"]["id"], fact.id);
    assert_eq!(later.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn restart_reconnect_retracts_previous_generation_before_new_snapshot() {
    let mut daemon = Daemon::start().await;
    let (mut subscriber, first_reset) = Peer::connect_with_reset(&daemon.socket).await;
    let mut client = SubscriptionClient::new();
    assert!(matches!(client.receive(&serde_json::to_vec(&first_reset).unwrap()).unwrap().as_slice(), [
        SubscriptionEvent::Reset { .. }
    ]));
    subscriber.subscribe(owned("before")).await;
    assert_eq!(subscriber.next().await["op"], "snapshot_end");
    let mut publisher = Peer::connect(&daemon.socket).await;
    let old = Fact::new("before", FactKind::Goal, "root/a", "building").unwrap();
    publisher.publish(&old).await;
    let old_frame = subscriber.next().await;
    assert!(matches!(client.receive(&serde_json::to_vec(&old_frame).unwrap()).unwrap().as_slice(), [
        SubscriptionEvent::Publish(_)
    ]));
    let old_generation = client.generation().unwrap().to_owned();
    daemon.restart().await;
    let mut eof = String::new();
    assert_eq!(tokio::time::timeout(WAIT, subscriber.read.read_line(&mut eof)).await.unwrap().unwrap(), 0);
    let (mut reconnected, reset) = Peer::connect_with_reset(&daemon.socket).await;
    let events = client.receive(&serde_json::to_vec(&reset).unwrap()).unwrap();
    assert!(
        matches!(events.as_slice(), [SubscriptionEvent::Retract { id }, SubscriptionEvent::Reset { .. }] if id == &old.id)
    );
    assert_ne!(client.generation(), Some(old_generation.as_str()));
    reconnected.subscribe(owned("after")).await;
    assert_eq!(reconnected.next().await["op"], "snapshot_end");
    let mut new_publisher = Peer::connect(&daemon.socket).await;
    let new_fact = Fact::new("after", FactKind::Goal, "root/b", "building").unwrap();
    new_publisher.publish(&new_fact).await;
    let new_frame = reconnected.next().await;
    assert_eq!(new_frame["fact"]["id"], new_fact.id);
    assert!(matches!(client.receive(&serde_json::to_vec(&new_frame).unwrap()).unwrap().as_slice(), [
        SubscriptionEvent::Publish(_)
    ]));
    assert_eq!(client.disconnected(), vec![SubscriptionEvent::Retract { id: new_fact.id }]);
}

#[tokio::test]
async fn active_daemon_socket_is_not_unlinked_by_competitor() {
    let daemon = Daemon::start().await;
    let competing = Command::new(env!("CARGO_BIN_EXE_mantle-coordination"))
        .args(["serve", "--socket"])
        .arg(&daemon.socket)
        .output()
        .unwrap();
    assert!(!competing.status.success());
    assert!(daemon.socket.exists());
    let mut client = Peer::connect(&daemon.socket).await;
    client.subscribe(owned("unused")).await;
    assert_eq!(client.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn slow_subscriber_is_dropped_without_blocking_publishers() {
    let daemon = Daemon::start().await;
    let slow = UnixStream::connect(&daemon.socket).await.unwrap();
    let (slow_reader, mut slow_writer) = slow.into_split();
    slow_writer
        .write_all(b"{\"op\":\"subscribe\",\"filter\":{\"owner\":\"flood\",\"kind\":null,\"subject_prefix\":null}}\n")
        .await
        .unwrap();
    let mut publisher = Peer::connect(&daemon.socket).await;
    for index in 0..1024 {
        let fact = Fact::new("flood", FactKind::Goal, "root", &format!("{index}:{}", "x".repeat(1000))).unwrap();
        publisher.publish(&fact).await;
    }
    let eof = tokio::time::timeout(WAIT, async {
        let mut buffer = [0_u8; 8192];
        loop {
            if slow_reader.readable().await.is_err() {
                return true;
            }
            if let Ok(0) = slow_reader.try_read(&mut buffer) {
                return true;
            }
        }
    })
    .await
    .expect("subscriber did not reach bounded drop");
    assert!(eof);
    let mut healthy = Peer::connect(&daemon.socket).await;
    healthy.subscribe(owned("flood")).await;
    assert_eq!(healthy.next().await["op"], "snapshot");
    assert_eq!(healthy.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn large_snapshot_stalls_only_its_slow_subscriber() {
    let daemon = Daemon::start().await;
    let mut publisher = Peer::connect(&daemon.socket).await;
    let body = "x".repeat(1300);
    for index in 0..320 {
        let fact = Fact::new("many", FactKind::Goal, &format!("root/{index}"), &body).unwrap();
        publisher.publish(&fact).await;
    }
    let mut slow = Peer::connect(&daemon.socket).await;
    slow.subscribe(owned("many")).await;
    assert_eq!(slow.next().await["op"], "snapshot");
    let changed = Fact::new("many", FactKind::Goal, "root/0", &format!("new:{body}")).unwrap();
    for index in 0..128 {
        let fact = Fact::new("many", FactKind::Goal, "root/0", &format!("{index}:{body}")).unwrap();
        publisher.publish(&fact).await;
    }
    publisher.publish(&changed).await;
    tokio::time::timeout(WAIT, async {
        let mut line = String::new();
        loop {
            line.clear();
            if slow.read.read_line(&mut line).await.unwrap() == 0 {
                break;
            }
        }
    })
    .await
    .expect("backpressured snapshot did not drop subscriber");
    let mut healthy = Peer::connect(&daemon.socket).await;
    healthy.subscribe(owned("many")).await;
    let mut seen_changed = false;
    for _ in 0..320 {
        let response = healthy.next().await;
        assert_eq!(response["op"], "snapshot");
        if response["fact"]["id"] == changed.id {
            assert_eq!(response["fact"]["state"], changed.state);
            seen_changed = true;
        }
    }
    assert!(seen_changed);
    assert_eq!(healthy.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn publisher_session_acknowledges_changes_and_retracts_on_drop() {
    let daemon = Daemon::start().await;
    let mut subscriber = Peer::connect(&daemon.socket).await;
    subscriber.subscribe(owned("client-owner")).await;
    assert_eq!(subscriber.next().await["op"], "snapshot_end");
    let mut publisher = PublisherSession::connect(&daemon.socket).await.unwrap();
    assert!(!publisher.generation().is_empty());
    let goal = Fact::new("client-owner", FactKind::Goal, "root/a", "building").unwrap();
    publisher.publish(&goal).await.unwrap();
    assert_eq!(subscriber.next().await["fact"]["id"], goal.id);
    publisher.retract(&goal.id).await.unwrap();
    assert_eq!(subscriber.next().await["id"], goal.id);
    let presence = Fact::new("client-owner", FactKind::Worker, "worker/a", "present").unwrap();
    publisher.publish(&presence).await.unwrap();
    assert_eq!(subscriber.next().await["fact"]["id"], presence.id);
    drop(publisher);
    assert_eq!(subscriber.next().await["id"], presence.id);
}

#[tokio::test]
async fn publisher_session_rejects_bad_fact_and_foreign_owner_without_overwriting() {
    let daemon = Daemon::start().await;
    let mut first = PublisherSession::connect(&daemon.socket).await.unwrap();
    let fact = readiness("one", "remote", ServiceState::Started);
    let mut forged = fact.clone();
    forged.id = "0".repeat(64);
    let error = first.publish(&forged).await.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    first.publish(&fact).await.unwrap();
    let mut second = PublisherSession::connect(&daemon.socket).await.unwrap();
    let error = second.publish(&fact).await.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    let error = second.retract(&fact.id).await.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("one")).await;
    assert_eq!(observer.next().await["fact"]["id"], fact.id);
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let absent = daemon.socket.with_file_name("no-coordination.sock");
    assert_eq!(PublisherSession::connect(&absent).await.err().unwrap().kind(), std::io::ErrorKind::NotFound);
}

#[tokio::test]
async fn blocking_session_reuses_socket_and_retracts_when_dropped() {
    let daemon = Daemon::start().await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("blocking")).await;
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let mut publisher = BlockingPublisherSession::connect(&daemon.socket).unwrap();
    assert!(!publisher.generation().is_empty());
    let started = readiness("blocking", "service", ServiceState::Started);
    let ready = readiness("blocking", "service", ServiceState::Ready);
    publisher.publish(&started).unwrap();
    publisher.publish(&ready).unwrap();
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Started);
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Ready);
    publisher.retract(&ready.id).unwrap();
    assert_eq!(observer.next().await["id"], ready.id);
    publisher.publish(&ready).unwrap();
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Ready);
    drop(publisher);
    let withdrawn = [observer.next().await, observer.next().await];
    assert!(withdrawn.iter().all(|event| event["op"] == "retract"));
    assert!(withdrawn.iter().any(|event| event["id"] == started.id));
    assert!(withdrawn.iter().any(|event| event["id"] == ready.id));
}

#[tokio::test]
async fn blocking_session_rejects_foreign_owner_and_missing_endpoint() {
    let daemon = Daemon::start().await;
    let mut first = BlockingPublisherSession::connect(&daemon.socket).unwrap();
    let fact = Fact::new("blocking-owner", FactKind::Worker, "worker/1", "present").unwrap();
    let mut invalid = fact.clone();
    invalid.id = "0".repeat(64);
    assert_eq!(first.publish(&invalid).unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    first.publish(&fact).unwrap();
    let mut second = BlockingPublisherSession::connect(&daemon.socket).unwrap();
    assert_eq!(second.publish(&fact).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(second.retract(&fact.id).unwrap_err().kind(), std::io::ErrorKind::BrokenPipe);
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("blocking-owner")).await;
    assert_eq!(observer.next().await["fact"]["id"], fact.id);
    assert_eq!(observer.next().await["op"], "snapshot_end");
    assert_eq!(
        BlockingPublisherSession::connect(&daemon.socket.with_file_name("absent.sock"))
            .err()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
}

#[tokio::test]
async fn background_publisher_delivers_started_then_ready_before_one_shot_exit() {
    let daemon = Daemon::start().await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("service")).await;
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let publisher = BestEffortPublisher::new(&daemon.socket).unwrap();
    let started = readiness("service", "cache", ServiceState::Started);
    let ready = readiness("service", "cache", ServiceState::Ready);
    publisher.try_publish(started.clone()).unwrap();
    publisher.try_publish(ready.clone()).unwrap();
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Started);
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Ready);
    assert!(publisher.finish(Duration::from_secs(5)), "one-shot publisher must drain ACKs");
    let withdrawn = [observer.next().await, observer.next().await];
    assert!(withdrawn.iter().any(|event| event["id"] == started.id));
    assert!(withdrawn.iter().any(|event| event["id"] == ready.id));
}

#[tokio::test]
async fn background_publisher_bounds_unreachable_and_nonreading_shutdown() {
    let directory = tempfile::tempdir().unwrap();
    let absent = directory.path().join("absent.sock");
    let missing = BestEffortPublisher::new(&absent).unwrap();
    let begin = std::time::Instant::now();
    assert!(!missing.finish(Duration::from_secs(5)));
    assert!(begin.elapsed() <= MAX_END_FLUSH + Duration::from_millis(100));

    let stalled = directory.path().join("stalled.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&stalled).unwrap();
    let publisher = BestEffortPublisher::new(&stalled).unwrap();
    let fact = readiness("stalled", "remote", ServiceState::Started);
    publisher.try_publish(fact).unwrap();
    let begin = std::time::Instant::now();
    assert!(!publisher.finish(Duration::from_secs(5)));
    assert!(begin.elapsed() <= MAX_END_FLUSH + Duration::from_millis(100));
}

#[tokio::test]
async fn background_publisher_full_queue_degrades_without_blocking_caller() {
    let directory = tempfile::tempdir().unwrap();
    let stalled = directory.path().join("stalled.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&stalled).unwrap();
    let publisher = BestEffortPublisher::new(&stalled).unwrap();
    let fact = Fact::new("flood", FactKind::Goal, "root", "building").unwrap();
    let mut result = Ok(());
    for index in 0..(crunch_live_state_core::MAX_PENDING_EVENTS + 2) {
        let queued = Fact::new("flood", FactKind::Goal, &format!("root/{index}"), "building").unwrap();
        result = publisher.try_publish(queued);
        if result.is_err() {
            break;
        }
    }
    assert_eq!(result, Err(BestEffortError::Full));
    assert!(publisher.is_degraded());
    assert_eq!(publisher.try_publish(fact), Err(BestEffortError::Unavailable));
}

#[tokio::test]
async fn daemon_readiness_starts_before_snapshot_then_readies_on_real_subscription() {
    let mut daemon = Daemon::start().await;
    let (mut observer, first_reset) = Peer::connect_with_reset(&daemon.socket).await;
    let mut client = SubscriptionClient::new();
    client.receive(&serde_json::to_vec(&first_reset).unwrap()).unwrap();
    observer
        .send(json!({
            "op":"subscribe",
            "filter":{"owner":null,"kind":null,"subject_prefix":"x".repeat(MAX_FILTER_BYTES)}
        }))
        .await;
    assert_eq!(observer.next().await["op"], "error");
    let filter = Filter {
        owner: None,
        kind: Some(FactKind::ServiceReadiness),
        subject_prefix: Some("service/coordination/".to_owned()),
    };
    observer.subscribe(filter.clone()).await;
    let started = observer.next().await;
    assert_eq!(started["op"], "snapshot");
    assert_eq!(asserted_state(&started), ServiceState::Started);
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let ready = observer.next().await;
    assert_eq!(ready["op"], "publish");
    assert_eq!(asserted_state(&ready), ServiceState::Ready);
    assert_ne!(started["fact"]["id"], ready["fact"]["id"]);
    assert_eq!(started["fact"]["owner"], ready["fact"]["owner"]);
    client.receive(&serde_json::to_vec(&started).unwrap()).unwrap();
    client.receive(&serde_json::to_vec(&ready).unwrap()).unwrap();
    daemon.restart().await;
    let (mut reconnected, reset) = Peer::connect_with_reset(&daemon.socket).await;
    let events = client.receive(&serde_json::to_vec(&reset).unwrap()).unwrap();
    let withdrawn: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            SubscriptionEvent::Retract { id } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(withdrawn.len(), 2);
    assert!(withdrawn.contains(&started["fact"]["id"].as_str().unwrap()));
    assert!(withdrawn.contains(&ready["fact"]["id"].as_str().unwrap()));
    assert!(matches!(events.last(), Some(SubscriptionEvent::Reset { .. })));
    reconnected.subscribe(filter).await;
    assert_eq!(asserted_state(&reconnected.next().await), ServiceState::Started);
    assert_eq!(reconnected.next().await["op"], "snapshot_end");
    assert_eq!(asserted_state(&reconnected.next().await), ServiceState::Ready);
}

#[tokio::test]
async fn rejects_raw_readiness_missing_policy_wrong_subject_and_premature_ready() {
    let daemon = Daemon::start().await;
    let mut publisher = Peer::connect(&daemon.socket).await;
    let started = readiness("typed", "api", ServiceState::Started);
    let ready = readiness("typed", "api", ServiceState::Ready);
    publisher.send(json!({"op":"publish","fact":ready})).await;
    assert_eq!(publisher.next().await["op"], "error");
    let mut raw = started.clone();
    raw.state = "started".to_owned();
    publisher.send(json!({"op":"publish","fact":raw})).await;
    assert_eq!(publisher.next().await["op"], "error");
    let mut missing = started.clone();
    let mut value: Value = serde_json::from_str(&missing.state).unwrap();
    value.as_object_mut().unwrap().remove("restart_policy");
    missing.state = value.to_string();
    publisher.send(json!({"op":"publish","fact":missing})).await;
    assert_eq!(publisher.next().await["op"], "error");
    let mut unknown = started.clone();
    let mut value: Value = serde_json::from_str(&unknown.state).unwrap();
    value["restart_policy"] = json!("sometimes");
    unknown.state = value.to_string();
    publisher.send(json!({"op":"publish","fact":unknown})).await;
    assert_eq!(publisher.next().await["op"], "error");
    let wrong_subject = Fact::new("typed", FactKind::ServiceReadiness, "service/api/ready", &started.state).unwrap();
    publisher.send(json!({"op":"publish","fact":wrong_subject})).await;
    assert_eq!(publisher.next().await["op"], "error");
    publisher.publish(&started).await;
    let mut changed_policy = ready.clone();
    let mut value: Value = serde_json::from_str(&changed_policy.state).unwrap();
    value["restart_policy"] = json!("always");
    changed_policy.state = value.to_string();
    publisher.send(json!({"op":"publish","fact":changed_policy})).await;
    assert_eq!(publisher.next().await["op"], "error");
    publisher.publish(&ready).await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("typed")).await;
    let snapshot = [observer.next().await, observer.next().await];
    assert!(snapshot.iter().any(|event| asserted_state(event) == ServiceState::Started));
    assert!(snapshot.iter().any(|event| asserted_state(event) == ServiceState::Ready));
    assert_eq!(observer.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn dependency_loss_retracts_ready_without_retracting_unrelated_started() {
    let daemon = Daemon::start().await;
    let mut publisher = Peer::connect(&daemon.socket).await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("dep-owner")).await;
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let database_started = readiness("dep-owner", "database", ServiceState::Started);
    let database_ready = readiness("dep-owner", "database", ServiceState::Ready);
    let dependent_blocked = readiness_with("dep-owner", "api", ServiceState::Started, &["database"], Some("database"));
    let dependent_started = readiness_with("dep-owner", "api", ServiceState::Started, &["database"], None);
    let dependent_ready = readiness_with("dep-owner", "api", ServiceState::Ready, &["database"], None);
    publisher.publish(&database_started).await;
    publisher.publish(&dependent_blocked).await;
    for _ in 0..2 {
        assert_eq!(observer.next().await["op"], "publish");
    }
    publisher.send(json!({"op":"publish","fact":dependent_ready})).await;
    assert_eq!(publisher.next().await["op"], "error");
    publisher.publish(&database_ready).await;
    publisher.publish(&dependent_started).await;
    publisher.publish(&dependent_ready).await;
    for _ in 0..3 {
        assert_eq!(observer.next().await["op"], "publish");
    }
    publisher.retract(&database_ready.id).await;
    let withdrawn = [observer.next().await, observer.next().await];
    assert!(withdrawn.iter().any(|event| event["id"] == database_ready.id));
    assert!(withdrawn.iter().any(|event| event["id"] == dependent_ready.id));
    let mut current = Peer::connect(&daemon.socket).await;
    current.subscribe(owned("dep-owner")).await;
    let snapshot = [current.next().await, current.next().await];
    assert!(snapshot.iter().any(|event| event["fact"]["id"] == database_started.id));
    assert!(snapshot.iter().any(|event| event["fact"]["id"] == dependent_started.id));
    assert_eq!(current.next().await["op"], "snapshot_end");
    publisher.retract(&database_started.id).await;
    assert_eq!(observer.next().await["id"], database_started.id);
    let failed = readiness_with("dep-owner", "api", ServiceState::Failed, &["database"], None);
    publisher.publish(&failed).await;
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Failed);
    assert_eq!(observer.next().await["id"], dependent_started.id);
    let mut after = Peer::connect(&daemon.socket).await;
    after.subscribe(owned("dep-owner")).await;
    assert_eq!(after.next().await["fact"]["id"], failed.id);
    assert_eq!(after.next().await["op"], "snapshot_end");
}

#[tokio::test]
async fn terminal_failure_and_completion_withdraw_active_service_states() {
    let daemon = Daemon::start().await;
    let mut observer = Peer::connect(&daemon.socket).await;
    observer.subscribe(owned("lifecycle")).await;
    assert_eq!(observer.next().await["op"], "snapshot_end");
    let mut publisher = Peer::connect(&daemon.socket).await;
    let failed_started = readiness("lifecycle", "failed_service", ServiceState::Started);
    let failed = readiness("lifecycle", "failed_service", ServiceState::Failed);
    publisher.publish(&failed_started).await;
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Started);
    publisher.publish(&failed).await;
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Failed);
    assert_eq!(observer.next().await["id"], failed_started.id);
    let completed_started = readiness("lifecycle", "successful_service", ServiceState::Started);
    let completed_ready = readiness("lifecycle", "successful_service", ServiceState::Ready);
    let completed = readiness("lifecycle", "successful_service", ServiceState::Complete);
    publisher.publish(&completed_started).await;
    publisher.publish(&completed_ready).await;
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Started);
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Ready);
    publisher.publish(&completed).await;
    assert_eq!(asserted_state(&observer.next().await), ServiceState::Complete);
    let withdrawn = [observer.next().await, observer.next().await];
    assert!(withdrawn.iter().any(|event| event["id"] == completed_started.id));
    assert!(withdrawn.iter().any(|event| event["id"] == completed_ready.id));
    let mut current = Peer::connect(&daemon.socket).await;
    current.subscribe(owned("lifecycle")).await;
    let remaining = [current.next().await, current.next().await];
    assert!(remaining.iter().any(|event| event["fact"]["id"] == failed.id));
    assert!(remaining.iter().any(|event| event["fact"]["id"] == completed.id));
    assert_eq!(current.next().await["op"], "snapshot_end");
}
