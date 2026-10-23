//! Real disk/lock races, using the existing coordinator admission constructors.
//! No remote execution result is manufactured by these tests.
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::mpsc;

use super::*;
use crate::remote_build::gateway::GatewayAttemptRecord;
use crate::remote_build::gateway::GatewayPhase;

fn gateway_record(public_byte: char, binding: Option<RemoteProductionAttemptBinding>) -> GatewayAttemptRecord {
    let request = fixture_derivation_request();
    // Gateway attempts admit Nix-client derivations, which live under /nix/store.
    let drv_path = fixture_derivation_drv_path(&request).replacen("/mantle/store/", "/nix/store/", 1);
    let mut record = GatewayAttemptRecord {
        public_id: public_byte.to_string().repeat(64),
        subject: "subject".into(),
        account: "account".into(),
        project: "project".into(),
        authority_policy: "policy".into(),
        authority_decision: "decision".into(),
        endpoint: "builder-1".into(),
        platform: DEFAULT_REMOTE_ACTION_SYSTEM.into(),
        transitions: Vec::new(),
        reuse: None,
        published_evidence: None,
        retries: Vec::new(),
        selection_json: None,
        request_digest: *blake3::hash(drv_path.as_bytes()).as_bytes(),
        drv_path,
        request_id: request.request_id,
        dispatch_started: binding.is_some(),
        binding,
        cancel_requested: false,
        local_teardown_confirmed: false,
        recovery_uncertain: false,
        result_evidence: None,
        outputs: Vec::new(),
        produced_outputs: Vec::new(),
        usage: None,
        public_log: Vec::new(),
    };
    record.transition(
        if record.dispatch_started {
            GatewayPhase::Running
        } else {
            GatewayPhase::Queued
        },
        TEST_ATTEMPT_NOW_UNIX_S,
        "gateway-save-race-admitted",
    );
    record
}

fn gateway_state() -> (tempfile::TempDir, String, RemoteProductionAttemptBinding) {
    let (temp, mut state, job_id) = persisted_log_fixture(&[], RemoteLogRetentionPolicy::default());
    let attempt = state.jobs[&job_id].current_attempt.as_ref().expect("real admitted attempt");
    let binding = RemoteProductionAttemptBinding {
        job_id,
        attempt_id: attempt.attempt_id.clone(),
        fence_generation: attempt.fence_generation,
    };
    let record = gateway_record('a', Some(binding.clone()));
    let public_id = record.public_id.clone();
    state.gateway_attempts.insert(public_id.clone(), record);
    save_coordinator_state(temp.path(), &state).expect("save running gateway state");
    (temp, public_id, binding)
}

#[cfg(target_os = "linux")]
fn measured_failure_usage(
    binding: &RemoteProductionAttemptBinding,
    request_id: &str,
) -> usage::RemoteSignedUsageReport {
    let mut command = std::process::Command::new("/bin/sh");
    command.args(["-c", "printf measured-failure >&2; exit 7"]);
    let completion = usage::RemoteUsageCollector::begin(binding, request_id, std::time::Duration::from_secs(10))
        .expect("create exact-child collector")
        .spawn(&mut command)
        .expect("spawn real failing child")
        .finish()
        .expect("measure and reap real child");
    assert_eq!(completion.status().code(), Some(7));
    assert_eq!(completion.stderr(), b"measured-failure");
    let digest = usage::failure_result_digest(binding, request_id);
    let report = completion
        .bind_result(&digest)
        .expect("bind actual failure measurements")
        .sign(&fixture_keypair())
        .expect("sign measured usage");
    report
        .verify(binding, request_id, &digest, &fixture_keypair().verifying_key)
        .expect("authenticate retained measurement");
    report
}

#[cfg(target_os = "linux")]
#[test]
fn stale_runner_save_cannot_erase_accepted_cancel_or_independent_records() {
    let (temp, id, binding) = gateway_state();
    let (loaded_tx, loaded_rx) = mpsc::sync_channel(0);
    let (resume_tx, resume_rx) = mpsc::sync_channel(0);
    let runner_dir = temp.path().to_path_buf();
    let runner_id = id.clone();
    let runner = std::thread::spawn(move || {
        let mut stale = load_coordinator_state(&runner_dir).expect("runner reads before network wait");
        loaded_tx.send(()).expect("signal stale read");
        resume_rx.recv().expect("wait until cancel durably commits");
        stale.gateway_attempts.get_mut(&runner_id).expect("runner record").transition(
            GatewayPhase::Failed,
            TEST_ATTEMPT_NOW_UNIX_S + 2,
            "gateway-stale-runner-failure",
        );
        save_coordinator_state(&runner_dir, &stale).expect_err("stale runner must conflict").to_string()
    });
    loaded_rx.recv().expect("runner holds old snapshot");
    let signed_usage = measured_failure_usage(&binding, &fixture_derivation_request().request_id);
    let accepted = {
        // Exercise the real cancel lock/save shape, without retaining its lock
        // while the runner waits or while joining another thread.
        let _guard = acquire_remote_coordinator_mutation_guard(temp.path()).expect("cancel mutation guard");
        let mut current = load_coordinator_state(temp.path()).expect("cancel reload");
        let record = current.gateway_attempts.get_mut(&id).expect("cancel record");
        record.cancel_requested = true;
        record.usage = Some(signed_usage.clone());
        record.append_event("gateway-cancellation-requested");
        record.transition(GatewayPhase::Failed, TEST_ATTEMPT_NOW_UNIX_S + 1, "gateway-independent-recovery");
        let independent = gateway_record('b', None);
        current.gateway_attempts.insert(independent.public_id.clone(), independent);
        save_coordinator_state(temp.path(), &current).expect("accept cancellation and independent record");
        fs::read(coordinator_state_path(temp.path())).expect("read accepted durable snapshot")
    };
    resume_tx.send(()).expect("release stale runner");
    assert!(runner.join().expect("runner completes").contains("remote-coordinator-save-conflict:"));
    assert_eq!(fs::read(coordinator_state_path(temp.path())).expect("read after stale save"), accepted);
    let mut reloaded = load_coordinator_state(temp.path()).expect("reload preserved state");
    assert!(gateway::cancellation_requested(&reloaded, &binding));
    assert_eq!(
        reloaded.gateway_attempts[&id].transitions.last().expect("recovery transition").reason,
        "gateway-independent-recovery"
    );
    assert!(reloaded.gateway_attempts.contains_key(&"b".repeat(64)));
    assert_eq!(reloaded.gateway_attempts[&id].usage.as_ref(), Some(&signed_usage));
    reloaded.gateway_attempts.get_mut(&id).expect("current record").local_teardown_confirmed = true;
    save_coordinator_state(temp.path(), &reloaded).expect("fresh fenced teardown can commit");
    assert!(
        load_coordinator_state(temp.path()).expect("reopen acknowledged state").gateway_attempts[&id].cancel_requested
    );
}

#[test]
fn simultaneous_saves_of_one_revision_have_exactly_one_winner() {
    let (temp, id, _) = gateway_state();
    let shared = load_coordinator_state(temp.path()).expect("read shared revision");
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = ["gateway-writer-one", "gateway-writer-two"]
        .into_iter()
        .map(|event| {
            let barrier = Arc::clone(&barrier);
            let state_dir = temp.path().to_path_buf();
            let mut candidate = shared.clone();
            let id = id.clone();
            std::thread::spawn(move || {
                candidate.gateway_attempts.get_mut(&id).expect("writer record").append_event(event);
                barrier.wait();
                (event, save_coordinator_state(&state_dir, &candidate))
            })
        })
        .collect();
    let results: Vec<_> = handles.into_iter().map(|thread| thread.join().expect("save contender exits")).collect();
    let winner = results.iter().find(|(_, result)| result.is_ok()).expect("one successful writer").0;
    let loser = results.iter().find(|(_, result)| result.is_err()).expect("one rejected writer");
    assert!(
        loser
            .1
            .as_ref()
            .expect_err("loser conflicts")
            .to_string()
            .contains("remote-coordinator-save-conflict:")
    );
    let current = load_coordinator_state(temp.path()).expect("reopen winner");
    assert!(current.gateway_attempts[&id].public_log.ends_with(format!("{winner}\n").as_bytes()));
    assert_eq!(current.persistence_revision.get(), shared.persistence_revision.get() + 1);
}

#[test]
fn unsuccessful_write_does_not_advance_revision_or_prevent_fresh_retry() {
    let (temp, id, _) = gateway_state();
    let mut current = load_coordinator_state(temp.path()).expect("load current revision");
    let revision = current.persistence_revision.get();
    let accepted = fs::read(coordinator_state_path(temp.path())).expect("read initial state");
    let temporary = coordinator_state_path(temp.path()).with_extension(TEMP_FILE_EXTENSION);
    fs::create_dir(&temporary).expect("make actual temp write fail");
    current.gateway_attempts.get_mut(&id).expect("current record").cancel_requested = true;
    assert!(save_coordinator_state(temp.path(), &current).is_err());
    assert_eq!(current.persistence_revision.get(), revision);
    assert_eq!(fs::read(coordinator_state_path(temp.path())).expect("read unchanged state"), accepted);
    fs::remove_dir(temporary).expect("remove failed-write obstruction");
    save_coordinator_state(temp.path(), &current).expect("same fresh snapshot retries after IO failure");
    let reopened = load_coordinator_state(temp.path()).expect("reopen accepted retry");
    assert_eq!(reopened.persistence_revision.get(), revision + 1);
    assert!(reopened.gateway_attempts[&id].cancel_requested);
}
