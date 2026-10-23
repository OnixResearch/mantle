use std::io::Cursor;
use std::os::unix::fs::OpenOptionsExt as _;

use crunch_remote_core::endpoint::EndpointBinding;
use crunch_remote_core::endpoint::EndpointBindingSpec;
use crunch_remote_core::endpoint::EndpointKind;

use super::*;

pub(super) fn descriptor(name: &str, prefix: &str) -> EndpointBinding {
    EndpointBinding {
        spec: EndpointBindingSpec {
            kind: EndpointKind::BuilderTicket,
            name: name.into(),
            scope: "fixture-builds".into(),
            trust_requirement: "fixture-builder-key".into(),
            store_prefix: prefix.into(),
            expires_unix_s: u64::MAX,
        },
        generation: 1,
    }
}

pub(super) fn provision(
    state_dir: &Path,
    builder: &RemoteLoopbackBuilder,
) -> endpoint_config::RemoteEndpointBindingConfig {
    let expected = descriptor(&builder.endpoint_id, &builder.store_prefix);
    let store = endpoint::EndpointBindingStore::open(state_dir).unwrap();
    let binding = store.create(expected.spec, None, crate::unix_time_now_s().unwrap()).unwrap();
    let config_path = state_dir.join("test-endpoint-config.json");
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(&config_path).unwrap();
    serde_json::to_writer(&mut file, &serde_json::json!({
        "schema": endpoint_config::REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA,
        "endpoint_state_directory": state_dir,
        "build": { "endpoint_binding": binding, "scope": "fixture-builds", "trust_requirement": "fixture-builder-key" },
        "source_bindings": []
    })).unwrap();
    drop(file);
    endpoint_config::RemoteEndpointBindingConfig::load(&config_path).unwrap()
}

#[test]
fn production_rejects_missing_forged_stale_and_revoked_binding_before_manifest_or_feedback() {
    for case in ["missing", "forged", "stale", "revoked", "wrong-kind", "wrong-trust"] {
        let root = tempfile::tempdir().unwrap();
        let builder = fixture_loopback_builder();
        let mut config = provision(root.path(), &builder);
        let mut presented = config.build_binding().unwrap().clone();
        let store = endpoint::EndpointBindingStore::open(root.path()).unwrap();
        match case {
            "forged" => presented.generation += 4,
            "stale" => {
                store.revoke(&presented).unwrap();
                let new = store
                    .create(presented.spec.clone(), Some(presented.generation), crate::unix_time_now_s().unwrap())
                    .unwrap();
                config.build.as_mut().unwrap().endpoint_binding = new;
            }
            "revoked" => store.revoke(&presented).unwrap(),
            "wrong-kind" => presented.spec.kind = EndpointKind::Transfer,
            "wrong-trust" => config.build.as_mut().unwrap().trust_requirement = "independent-untrusted-key".into(),
            _ => {}
        }
        let mut client = fixture_loopback_client(vec!["input-a".into()], vec!["builder-key".into()]);
        client.request.endpoint_binding = (case != "missing").then_some(presented);
        client.request.transfer_policy = Some(RemoteTransferPolicy::default());
        let prefix = encode_frame_stream(&[
            RemoteFrame::Hello { hello: client.hello },
            RemoteFrame::AuthTicket { auth: client.auth },
            RemoteFrame::BuildRequest {
                request: client.request,
            },
        ]);
        let mut bytes = prefix.clone();
        bytes.extend_from_slice(b"NOT-A-MANIFEST\n");
        let mut reader = Cursor::new(bytes);
        let mut writer = Vec::new();
        let key = fixture_ticket_verifier_key();
        let executor = fixture_local_build_executor();
        let context = RemoteProductionServerContext {
            builder: &builder,
            verifier_key: &key,
            executor: &executor,
            secret_request: None,
            usage_cgroup_root: None,
            credential_state_dir: root.path(),
            binding_config: Some(&config),
            execution_state_dir: root.path(),
            authenticated_client_endpoint: None,
        };
        let mut called = false;
        assert!(
            serve_stdio_remote_production_once_with_commit(&mut reader, &mut writer, &context, &mut |_, _, _, _| {
                called = true;
                Err("must-not-commit".into())
            })
            .is_err(),
            "{case}"
        );
        assert!(!called, "{case}");
        assert!(writer.is_empty(), "{case}");
        assert_eq!(reader.position(), prefix.len() as u64, "{case}: manifest was consumed");
    }
}

#[test]
fn production_shared_binding_lease_allows_independent_ticket_commit_once() {
    let root = tempfile::tempdir().unwrap();
    let builder = fixture_loopback_builder();
    let config = provision(root.path(), &builder);
    let now = crate::unix_time_now_s().unwrap();
    let key = fixture_ticket_verifier_key();
    let mut ticket = fixture_ticket();
    // Durable ticket state admits only generated IDs; this commit has no
    // authenticated peer, so the ticket is not endpoint-bound.
    ticket.id = "c".repeat(32);
    ticket.bound_client_endpoint = None;
    ticket.created_unix_s = now - 1;
    ticket.expires_unix_s = now + 3600;
    let ticket_id = ticket.id.clone();
    let mut state = RemoteTicketState::default();
    state.tickets.insert(ticket_id.clone(), ticket);
    state.bind_active_verifier_key(&key);
    {
        let _lock = crate::remote_credential_state::acquire_ticket_state_mutation_guard(root.path()).unwrap();
        save_ticket_state(root.path(), &state).unwrap();
    }
    let mut request = fixture_derivation_request();
    request.endpoint_binding = Some(config.build_binding().unwrap().clone());
    let admission = config.admit_build(request.endpoint_binding.as_ref().unwrap(), now, &builder.store_prefix).unwrap();
    let mut auth = fixture_loopback_client(Vec::new(), Vec::new()).auth;
    auth.ticket_id = ticket_id.clone();
    let mut effects = crunch_remote_core::effect::EffectSession::new(&request.request_id).unwrap();
    let before = state.tickets[&ticket_id].uses_remaining;
    commit_remote_ticket_admission_for_state(root.path(), &key, None, &auth, &request, &mut effects).unwrap();
    assert_eq!(load_ticket_state(root.path()).unwrap().tickets[&ticket_id].uses_remaining, before - 1);
    drop(admission);
    let _fresh = crate::remote_credential_state::acquire_ticket_state_mutation_guard(root.path()).unwrap();
}

struct Daemon {
    socket: PathBuf,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}
impl Daemon {
    fn start(root: &Path) -> Self {
        let socket = root.join("live.sock");
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let owned_socket = socket.clone();
        let state = root.to_path_buf();
        let flag = stop.clone();
        let thread = std::thread::spawn(move || live_state::daemon::serve_with_state(&owned_socket, &state, &flag));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::os::unix::net::UnixStream::connect(&socket).is_err() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Self {
            socket,
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for Daemon {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        self.thread.take().unwrap().join().unwrap().unwrap();
    }
}

#[test]
fn actual_daemon_resolves_retracts_recreates_and_persists() {
    use endpoint_service::ENDPOINT_PROTOCOL_SCHEMA as SCHEMA;
    use endpoint_service::Reply;
    use endpoint_service::Request;
    let root = tempfile::tempdir().unwrap();
    let binding;
    {
        let daemon = Daemon::start(root.path());
        let spec = descriptor("builder-1", "/mantle/store").spec;
        let Reply::Resolved { binding: old, .. } = endpoint_service::exchange(&daemon.socket, &Request::Create {
            schema: SCHEMA.into(),
            spec,
            expected_previous_generation: None,
        })
        .unwrap() else {
            panic!("expected descriptor")
        };
        for _ in 0..2 {
            let Reply::Resolved { binding: resolved, .. } =
                endpoint_service::exchange(&daemon.socket, &Request::Resolve {
                    schema: SCHEMA.into(),
                    binding: old.clone(),
                })
                .unwrap()
            else {
                panic!("expected resolution")
            };
            assert_eq!(resolved, old);
        }
        endpoint_service::exchange(&daemon.socket, &Request::Revoke {
            schema: SCHEMA.into(),
            binding: old.clone(),
        })
        .unwrap();
        assert_eq!(
            endpoint_service::exchange(&daemon.socket, &Request::Resolve {
                schema: SCHEMA.into(),
                binding: old.clone()
            })
            .unwrap_err()
            .kind(),
            std::io::ErrorKind::UnexpectedEof
        );
        let Reply::Resolved { binding: new, .. } = endpoint_service::exchange(&daemon.socket, &Request::Create {
            schema: SCHEMA.into(),
            spec: old.spec.clone(),
            expected_previous_generation: Some(old.generation),
        })
        .unwrap() else {
            panic!("expected replacement")
        };
        assert_eq!(new.generation, old.generation + 1);
        assert!(
            endpoint_service::exchange(&daemon.socket, &Request::Resolve {
                schema: SCHEMA.into(),
                binding: old
            })
            .is_err()
        );
        binding = new;
    }
    let daemon = Daemon::start(root.path());
    let Reply::Resolved { binding: resolved, .. } = endpoint_service::exchange(&daemon.socket, &Request::Resolve {
        schema: SCHEMA.into(),
        binding: binding.clone(),
    })
    .unwrap() else {
        panic!("expected durable resolution")
    };
    assert_eq!(resolved, binding);
}
