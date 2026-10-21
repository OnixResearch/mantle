//! Local deterministic fixtures only: these do not exercise a VMM, prove UCAN
//! verification, or establish production host dispatch correctness.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use parking_lot::Mutex;

use super::*;

const DRV: &str = "/nix/store/00000000000000000000000000000000-fixture.drv";
const OTHER_DRV: &str = "/nix/store/11111111111111111111111111111111-other.drv";

#[derive(Clone)]
struct Session {
    subject: String,
    scope: String,
    allowed: Vec<core::Permission>,
    valid: bool,
}

fn session(allowed: &[core::Permission]) -> Session {
    Session {
        subject: "fixture-owner".into(),
        scope: "fixture-project".into(),
        allowed: allowed.to_vec(),
        valid: true,
    }
}

fn authority(session: &Session) -> Authority<'_> {
    Authority {
        kind: core::CredentialKind::VerifiedCompatibilityTicket,
        subject: &session.subject,
        scope: &session.scope,
        audience: "fixture-service",
        expires_unix_s: 10_000,
        permissions: session.allowed.iter().fold(0, |bits, permission| bits | permission.bit()),
    }
}

fn full_session() -> Session {
    session(&[
        core::Permission::Build,
        core::Permission::StatusRead,
        core::Permission::LogRead,
        core::Permission::EventRead,
        core::Permission::Cancel,
        core::Permission::ResultRead,
        core::Permission::UsageRead,
    ])
}

#[derive(Default)]
struct HostState {
    attempts: BTreeMap<String, Observation>,
    cancellations: BTreeSet<String>,
    fail_submit_once: bool,
    verified_results: u32,
    oversized_log: bool,
}

#[derive(Clone, Default)]
struct Host(Arc<Mutex<HostState>>);

impl Authorization for Session {
    fn facts(&self) -> Authority<'_> {
        authority(self)
    }
}

impl Services for Host {
    type Session = Session;
    type Authorization = Session;
    async fn authenticate<'a>(
        &'a self,
        session: &'a Session,
        _now: u64,
        _request: Request<'a>,
    ) -> Result<Self::Authorization, Error> {
        if !session.valid {
            return Err(core::Reject::Authority.into());
        }
        Ok(session.clone())
    }
    async fn submit(
        &self,
        _session: &Self::Session,
        _authority: &Self::Authorization,
        id: &str,
        _drv: &str,
        _digest: [u8; 32],
    ) -> Result<Observation, Error> {
        let mut host = self.0.lock();
        let observation = host
            .attempts
            .entry(id.into())
            .or_insert(Observation {
                phase: Phase::Queued,
                fence: 7,
                result_evidence: None,
            })
            .clone();
        if host.fail_submit_once {
            host.fail_submit_once = false;
            return Err(Error::Host);
        }
        Ok(observation)
    }
    async fn observe(&self, _authority: &Self::Authorization, id: &str) -> Result<Observation, Error> {
        self.0.lock().attempts.get(id).cloned().ok_or(Error::Unknown)
    }
    async fn cancel(&self, _authority: &Self::Authorization, id: &str, fence: u64) -> Result<(), Error> {
        let mut host = self.0.lock();
        let current = host.attempts.get_mut(id).ok_or(Error::Unknown)?;
        if current.fence != fence {
            return Err(core::Reject::StaleFence.into());
        }
        if !current.phase.terminal() {
            current.phase = Phase::Cancelled;
        }
        host.cancellations.insert(id.into());
        Ok(())
    }
    async fn logs(
        &self,
        _authority: &Self::Authorization,
        _id: &str,
        offset: u64,
        maximum: u32,
    ) -> Result<Vec<u8>, Error> {
        if self.0.lock().oversized_log {
            return Ok(vec![b'x'; maximum as usize + 1]);
        }
        let bytes = b"public scrubbed log\n";
        if offset > bytes.len() as u64 {
            return Err(Error::Cursor);
        }
        Ok(bytes.iter().skip(offset as usize).take(maximum as usize).copied().collect())
    }
    async fn verify_result(
        &self,
        _session: &Self::Session,
        _authority: &Self::Authorization,
        _id: &str,
        evidence: [u8; 32],
    ) -> Result<(), Error> {
        if evidence != [9; 32] {
            return Err(Error::Integrity);
        }
        self.0.lock().verified_results += 1;
        Ok(())
    }
    async fn usage(
        &self,
        _session: &Self::Session,
        _authority: &Self::Authorization,
        _items: u32,
    ) -> Result<Usage, Error> {
        let host = self.0.lock();
        let completed = host.attempts.values().filter(|state| state.phase.terminal()).count() as u64;
        let admitted = host.attempts.len() as u64;
        Ok(Usage {
            admitted,
            active: admitted - completed,
            completed,
            cpu_millis: 0,
            output_bytes: 0,
            accounting: None,
        })
    }
}

fn directory() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    directory
}

fn open(directory: &Path, host: Host) -> Api<Host> {
    Api::open(directory, [4; 32], SigningKey::from_bytes(&[5; 32]), "fixture-service", [6; 32], host).unwrap()
}

async fn submit(api: &Api<Host>, session: &Session, key: &str) -> String {
    match api
        .handle(1, session, 100, Request::Submit {
            drv_path: DRV,
            idempotency_key: key,
        })
        .await
        .unwrap()
    {
        Response::Submitted { attempt_id, .. } => attempt_id,
        response => panic!("unexpected response {response:?}"),
    }
}

#[tokio::test]
async fn durable_duplicate_reconnect_and_pending_dispatch_recovery() {
    let directory = directory();
    let host = Host::default();
    host.0.lock().fail_submit_once = true;
    let api = open(directory.path(), host.clone());
    let session = full_session();
    assert_eq!(
        api.handle(1, &session, 100, Request::Submit {
            drv_path: DRV,
            idempotency_key: "secret-idempotency-material"
        })
        .await,
        Err(Error::Host)
    );
    drop(api);
    let api = open(directory.path(), host.clone());
    let id = submit(&api, &session, "secret-idempotency-material").await;
    assert_eq!(
        api.handle(1, &session, 100, Request::Submit {
            drv_path: DRV,
            idempotency_key: "secret-idempotency-material"
        })
        .await,
        Ok(Response::Submitted {
            attempt_id: id.clone(),
            recovered: true
        })
    );
    assert_eq!(host.0.lock().attempts.len(), 1);
    assert!(matches!(
        api.handle(1, &session, 100, Request::Status { attempt_id: &id }).await,
        Ok(Response::Status(Status {
            observation: Observation { fence: 7, .. },
            ..
        }))
    ));
    assert_eq!(
        api.handle(1, &session, 100, Request::Submit {
            drv_path: OTHER_DRV,
            idempotency_key: "secret-idempotency-material"
        })
        .await,
        Err(core::Reject::Conflict.into())
    );
    let bytes = std::fs::read(directory.path().join("gateway-api-v1.journal")).unwrap();
    assert!(
        !bytes
            .windows(b"secret-idempotency-material".len())
            .any(|part| part == b"secret-idempotency-material")
    );
}

#[tokio::test]
async fn completion_delivery_linkage_and_signature_tampering() {
    let directory = directory();
    let host = Host::default();
    let api = open(directory.path(), host.clone());
    let session = full_session();
    let id = submit(&api, &session, "complete").await;
    host.0.lock().attempts.get_mut(&id).unwrap().phase = Phase::Running;
    api.handle(1, &session, 100, Request::Status { attempt_id: &id }).await.unwrap();
    *host.0.lock().attempts.get_mut(&id).unwrap() = Observation {
        phase: Phase::Succeeded,
        fence: 7,
        result_evidence: Some([9; 32]),
    };
    let read = || Request::Events {
        attempt_id: &id,
        cursor: None,
        items: 16,
    };
    let Response::Events(first) = api.handle(1, &session, 100, read()).await.unwrap() else {
        panic!()
    };
    let Response::Events(second) = api.handle(1, &session, 100, read()).await.unwrap() else {
        panic!()
    };
    assert_eq!(first.values, second.values);
    assert_eq!(first.values.iter().map(|event| event.sequence).collect::<Vec<_>>(), [1, 2, 3]);
    let completion = first.values[2].completion.clone().unwrap();
    verify_completion(&completion, &api.verifying_key()).unwrap();
    assert_eq!(
        api.handle(1, &session, 100, Request::Result { attempt_id: &id }).await,
        Ok(Response::Result(Some(completion.clone())))
    );
    assert_eq!(host.0.lock().verified_results, 1);
    let mut tampered = completion.clone();
    tampered.result_evidence = Some([8; 32]);
    assert_eq!(verify_completion(&tampered, &api.verifying_key()), Err(Error::Integrity));
    let mut tampered = completion.clone();
    tampered.signature[0] ^= 1;
    assert_eq!(verify_completion(&tampered, &api.verifying_key()), Err(Error::Integrity));
    assert_eq!(
        verify_completion(&completion, &SigningKey::from_bytes(&[8; 32]).verifying_key()),
        Err(Error::Integrity)
    );
    drop(api);
    let api = open(directory.path(), host);
    let Response::Events(reconnected) = api.handle(1, &session, 101, read()).await.unwrap() else {
        panic!()
    };
    assert_eq!(reconnected.values, first.values);
}

#[tokio::test]
async fn cursor_scope_purpose_expiry_and_response_bounds() {
    let directory = directory();
    let host = Host::default();
    let api = open(directory.path(), host.clone());
    let session = full_session();
    let id = submit(&api, &session, "cursor").await;
    let other_id = submit(&api, &session, "other-cursor").await;
    let Response::Logs(first) = api
        .handle(1, &session, 100, Request::Logs {
            attempt_id: &id,
            cursor: None,
            bytes: 7,
        })
        .await
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(first.bytes, b"public ");
    let Response::Logs(next) = api
        .handle(1, &session, 101, Request::Logs {
            attempt_id: &id,
            cursor: Some(&first.next_cursor),
            bytes: 8,
        })
        .await
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(next.bytes, b"scrubbed");
    for (attempt_id, now) in [(&other_id, 101), (&id, 100 + CURSOR_TTL_SECS)] {
        assert_eq!(
            api.handle(1, &session, now, Request::Logs {
                attempt_id,
                cursor: Some(&first.next_cursor),
                bytes: 7
            })
            .await,
            Err(Error::Cursor)
        );
    }
    assert_eq!(
        api.handle(1, &session, 101, Request::Events {
            attempt_id: &id,
            cursor: Some(&first.next_cursor),
            items: 7
        })
        .await,
        Err(Error::Cursor)
    );
    let mut forged = first.next_cursor.into_bytes();
    forged[4] = if forged[4] == b'A' { b'B' } else { b'A' };
    let forged = String::from_utf8(forged).unwrap();
    assert_eq!(
        api.handle(1, &session, 101, Request::Logs {
            attempt_id: &id,
            cursor: Some(&forged),
            bytes: 7
        })
        .await,
        Err(Error::Cursor)
    );
    host.0.lock().oversized_log = true;
    assert_eq!(
        api.handle(1, &session, 101, Request::Logs {
            attempt_id: &id,
            cursor: None,
            bytes: 7
        })
        .await,
        Err(Error::Integrity)
    );
    assert_eq!(
        api.handle(1, &session, 101, Request::Logs {
            attempt_id: &id,
            cursor: None,
            bytes: core::MAX_LOG_WINDOW_BYTES + 1
        })
        .await,
        Err(core::Reject::Bound.into())
    );
    assert_eq!(
        api.handle(1, &session, 101, Request::Events {
            attempt_id: &id,
            cursor: None,
            items: 0
        })
        .await,
        Err(core::Reject::Bound.into())
    );
    assert_eq!(
        api.handle(1, &session, 101, Request::Usage {
            items: MAX_USAGE_ITEMS + 1
        })
        .await,
        Err(core::Reject::Bound.into())
    );
}

#[tokio::test]
async fn authority_unknown_cancellation_and_terminal_races() {
    let directory = directory();
    let host = Host::default();
    let api = open(directory.path(), host.clone());
    let owner = full_session();
    let build_only = session(&[core::Permission::Build]);
    let id = submit(&api, &owner, "cancel").await;
    assert_eq!(
        api.handle(1, &build_only, 100, Request::Status { attempt_id: &id }).await,
        Err(core::Reject::Authority.into())
    );
    let mut foreign = owner.clone();
    foreign.scope = "other-project".into();
    assert_eq!(
        api.handle(1, &foreign, 100, Request::Status { attempt_id: &id }).await,
        Err(core::Reject::NotOwner.into())
    );
    assert_eq!(
        api.handle(1, &owner, 100, Request::Status {
            attempt_id: "not-found"
        })
        .await,
        Err(Error::Unknown)
    );
    assert_eq!(
        api.handle(2, &owner, 100, Request::Status { attempt_id: &id }).await,
        Err(core::Reject::Version.into())
    );
    assert_eq!(
        api.handle(1, &owner, 100, Request::Cancel {
            attempt_id: &id,
            fence: 6
        })
        .await,
        Err(core::Reject::StaleFence.into())
    );
    for _ in 0..2 {
        api.handle(1, &owner, 100, Request::Cancel {
            attempt_id: &id,
            fence: 7,
        })
        .await
        .unwrap();
    }
    assert_eq!(host.0.lock().cancellations, BTreeSet::from([id.clone()]));
    assert!(matches!(
        api.handle(1, &owner, 100, Request::Status { attempt_id: &id }).await,
        Ok(Response::Status(Status {
            observation: Observation {
                phase: Phase::Cancelled,
                ..
            },
            ..
        }))
    ));
    assert_eq!(
        api.handle(1, &owner, 100, Request::Cancel {
            attempt_id: &id,
            fence: 7
        })
        .await,
        Ok(Response::CancelRequested {
            attempt_id: id.clone(),
            terminal: true
        })
    );
    host.0.lock().attempts.get_mut(&id).unwrap().phase = Phase::Succeeded;
    assert_eq!(api.handle(1, &owner, 100, Request::Status { attempt_id: &id }).await, Err(Error::Conflict));
    let mut invalid = owner;
    invalid.valid = false;
    assert_eq!(api.handle(1, &invalid, 100, Request::Usage { items: 1 }).await, Err(core::Reject::Authority.into()));
}

#[tokio::test]
async fn damaged_deleted_and_wrong_key_journals_fail_closed() {
    let directory = directory();
    let host = Host::default();
    let api = open(directory.path(), host.clone());
    submit(&api, &full_session(), "durable").await;
    drop(api);
    assert!(matches!(
        Api::open(
            directory.path(),
            [8; 32],
            SigningKey::from_bytes(&[5; 32]),
            "fixture-service",
            [6; 32],
            host.clone()
        ),
        Err(Error::Integrity)
    ));
    assert!(matches!(
        Api::open(directory.path(), [4; 32], SigningKey::from_bytes(&[5; 32]), "other-service", [6; 32], host.clone()),
        Err(Error::Integrity)
    ));
    assert!(matches!(
        Api::open(
            directory.path(),
            [4; 32],
            SigningKey::from_bytes(&[5; 32]),
            "fixture-service",
            [7; 32],
            host.clone()
        ),
        Err(Error::Integrity)
    ));
    assert!(matches!(
        Api::open(
            directory.path(),
            [4; 32],
            SigningKey::from_bytes(&[7; 32]),
            "fixture-service",
            [6; 32],
            host.clone()
        ),
        Err(Error::Integrity)
    ));
    let path = directory.path().join("gateway-api-v1.journal");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[40] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    assert!(matches!(
        Api::open(
            directory.path(),
            [4; 32],
            SigningKey::from_bytes(&[5; 32]),
            "fixture-service",
            [6; 32],
            host.clone()
        ),
        Err(Error::Integrity)
    ));
    std::fs::remove_file(path).unwrap();
    assert!(matches!(
        Api::open(directory.path(), [4; 32], SigningKey::from_bytes(&[5; 32]), "fixture-service", [6; 32], host),
        Err(Error::Storage)
    ));
}

#[test]
fn cursor_rejects_subject_scope_version_and_oversized_material() {
    let owner = full_session();
    let facts = authority(&owner);
    let cursor = crypto::cursor_write(&[4; 32], facts, "attempt", b'e', 12, 100).unwrap();
    assert_eq!(crypto::cursor_read(&[4; 32], Some(&cursor), facts, "attempt", b'e', 101), Ok(12));
    let mut other = owner.clone();
    other.subject = "other-owner".into();
    assert_eq!(
        crypto::cursor_read(&[4; 32], Some(&cursor), authority(&other), "attempt", b'e', 101),
        Err(Error::Cursor)
    );
    other = owner.clone();
    other.scope = "other-scope".into();
    assert_eq!(
        crypto::cursor_read(&[4; 32], Some(&cursor), authority(&other), "attempt", b'e', 101),
        Err(Error::Cursor)
    );
    assert_eq!(crypto::cursor_read(&[8; 32], Some(&cursor), facts, "attempt", b'e', 101), Err(Error::Cursor));
    let oversized = "x".repeat(core::MAX_CURSOR_BYTES as usize + 1);
    assert_eq!(crypto::cursor_read(&[4; 32], Some(&oversized), facts, "attempt", b'e', 101), Err(Error::Cursor));
    assert_eq!(
        crypto::cursor_read(&[4; 32], Some("not-a-versioned-cursor"), facts, "attempt", b'e', 101),
        Err(Error::Cursor)
    );
}

#[test]
fn event_budget_preserves_a_terminal_slot_and_rejects_fence_rollback() {
    let owner = full_session();
    let mut record = Record::pending("attempt".into(), authority(&owner), [1; 32], [2; 32]);
    let signer = SigningKey::from_bytes(&[5; 32]);
    for fence in 1..MAX_EVENTS_PER_ATTEMPT as u64 {
        record
            .observe(
                Observation {
                    phase: Phase::Running,
                    fence,
                    result_evidence: None,
                },
                &signer,
            )
            .unwrap();
    }
    assert_eq!(
        record.observe(
            Observation {
                phase: Phase::Running,
                fence: 1,
                result_evidence: None
            },
            &signer
        ),
        Err(Error::Integrity)
    );
    assert_eq!(
        record.observe(
            Observation {
                phase: Phase::Running,
                fence: 64,
                result_evidence: None
            },
            &signer
        ),
        Err(core::Reject::Bound.into())
    );
    record
        .observe(
            Observation {
                phase: Phase::Failed,
                fence: 64,
                result_evidence: None,
            },
            &signer,
        )
        .unwrap();
    let completion = record.events.last().unwrap().completion.as_ref().unwrap();
    assert_eq!(completion.sequence, MAX_EVENTS_PER_ATTEMPT as u64);
    assert_eq!(completion.terminal, Phase::Failed);
    verify_completion(completion, &signer.verifying_key()).unwrap();
}

#[tokio::test]
async fn status_read_permission_does_not_grant_usage_accounting() {
    let directory = directory();
    let host = Host::default();
    let api = open(directory.path(), host);
    let reader = session(&[core::Permission::StatusRead]);
    assert_eq!(api.handle(1, &reader, 100, Request::Usage { items: 1 }).await, Err(core::Reject::Authority.into()));
}
