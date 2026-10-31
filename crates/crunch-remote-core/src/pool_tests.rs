//! Behavior fixtures for explicit worker-pool admission.

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::endpoint::EndpointBinding;
use crate::endpoint::EndpointBindingError;
use crate::endpoint::EndpointBindingRecord;
use crate::endpoint::EndpointBindingSpec;
use crate::endpoint::EndpointBindingTable;
use crate::endpoint::EndpointKind;
use crate::pool::AdmittedPool;
use crate::pool::CallerCredential;
use crate::pool::DeclaredCapabilities;
use crate::pool::PoolAdmissionError;
use crate::pool::PoolEndpoint;
use crate::pool::PoolSelection;
use crate::pool::PoolSessionPolicy;
use crate::pool::TrustedOutputKey;
use crate::pool::admits_pool;

const NOW_UNIX_S: u64 = 1_000;
const EXPIRES_UNIX_S: u64 = 10_000;
const ENDPOINT_NAMES: [&str; 2] = ["worker-a", "worker-b"];

fn spec(name: &str) -> EndpointBindingSpec {
    EndpointBindingSpec {
        kind: EndpointKind::BuilderTicket,
        name: name.to_string(),
        scope: String::from("native-build"),
        trust_requirement: String::from("signed-content"),
        store_prefix: String::from("/mantle/store"),
        expires_unix_s: EXPIRES_UNIX_S,
    }
}

fn binding(name: &str) -> EndpointBinding {
    EndpointBinding {
        spec: spec(name),
        generation: 1,
    }
}

fn bindings(names: &[&str]) -> EndpointBindingTable {
    let records = names
        .iter()
        .map(|name| EndpointBindingRecord {
            binding: binding(name),
            revoked: false,
        })
        .collect::<Vec<_>>();
    EndpointBindingTable::from_records(records).expect("fixture bindings are valid")
}

fn capabilities() -> DeclaredCapabilities {
    DeclaredCapabilities {
        system: String::from("x86_64-linux"),
        sandbox_mode: String::from("practical"),
        network_mode: String::from("none"),
        features: vec![String::from("native-compile")],
    }
}

fn endpoint(name: &str) -> PoolEndpoint {
    PoolEndpoint {
        presented_binding: binding(name),
        transport: String::from("stdio"),
        capabilities: capabilities(),
        trusted_output_keys: vec![TrustedOutputKey {
            name: format!("{name}-1"),
            material_base64: format!("material-{name}"),
        }],
        credential_reference: format!("bearer-{name}"),
        declares_resource_inventory: true,
    }
}

fn selection(names: &[&str]) -> PoolSelection {
    PoolSelection {
        pool_id: String::from("native-pool"),
        endpoints: names.iter().map(|name| endpoint(name)).collect(),
        session: PoolSessionPolicy {
            max_concurrency: 2,
            job_count_max: 64,
            session_bytes_max: 1_048_576,
        },
    }
}

fn credentials(names: &[&str]) -> Vec<CallerCredential> {
    names
        .iter()
        .map(|name| CallerCredential {
            endpoint_name: name.to_string(),
            reference: format!("bearer-{name}"),
        })
        .collect()
}

fn admit(
    selection: &PoolSelection,
    table_names: &[&str],
    credential_names: &[&str],
) -> Result<AdmittedPool, PoolAdmissionError> {
    let table = bindings(table_names);
    let caller = credentials(credential_names);
    admits_pool(selection, &table, &caller, NOW_UNIX_S)
}

#[test]
fn two_independently_bound_workers_admit_together() {
    let selection = selection(&ENDPOINT_NAMES);
    let admitted = admit(&selection, &ENDPOINT_NAMES, &ENDPOINT_NAMES).expect("pool admission");
    assert_eq!(admitted.pool_id(), "native-pool");
    assert_eq!(admitted.endpoint_names(), &[String::from("worker-a"), String::from("worker-b")]);
    assert_eq!(admitted.session(), selection.session);
}

#[test]
fn pool_label_is_not_authority() {
    let selection = selection(&ENDPOINT_NAMES);
    assert_eq!(
        admit(&selection, &[], &ENDPOINT_NAMES),
        Err(PoolAdmissionError::EndpointBinding(EndpointBindingError::UnknownBinding))
    );
    assert_eq!(admit(&selection, &ENDPOINT_NAMES, &[]), Err(PoolAdmissionError::MissingCallerCredential));
    let mut revoked = bindings(&ENDPOINT_NAMES);
    revoked.revoke(&binding("worker-a")).expect("fixture revocation");
    let caller = credentials(&ENDPOINT_NAMES);
    assert_eq!(
        admits_pool(&selection, &revoked, &caller, NOW_UNIX_S),
        Err(PoolAdmissionError::EndpointBinding(EndpointBindingError::Revoked))
    );
}

#[test]
fn expired_binding_is_not_current_authority() {
    let selection = selection(&ENDPOINT_NAMES);
    let table = bindings(&ENDPOINT_NAMES);
    let caller = credentials(&ENDPOINT_NAMES);
    assert_eq!(
        admits_pool(&selection, &table, &caller, EXPIRES_UNIX_S),
        Err(PoolAdmissionError::EndpointBinding(EndpointBindingError::Expired))
    );
}

#[test]
fn duplicated_endpoint_or_bearer_is_not_a_pool() {
    let twice = selection(&["worker-a", "worker-a"]);
    assert_eq!(admit(&twice, &["worker-a"], &["worker-a"]), Err(PoolAdmissionError::DuplicateEndpoint));
    let mut shared = selection(&ENDPOINT_NAMES);
    shared.endpoints[1].credential_reference = String::from("bearer-worker-a");
    let caller = vec![
        CallerCredential {
            endpoint_name: String::from("worker-a"),
            reference: String::from("bearer-worker-a"),
        },
        CallerCredential {
            endpoint_name: String::from("worker-b"),
            reference: String::from("bearer-worker-a"),
        },
    ];
    let table = bindings(&ENDPOINT_NAMES);
    assert_eq!(
        admits_pool(&shared, &table, &caller, NOW_UNIX_S),
        Err(PoolAdmissionError::DuplicateCredentialReference)
    );
}

#[test]
fn output_trust_must_be_separate_and_unambiguous() {
    let mut untrusted = selection(&ENDPOINT_NAMES);
    untrusted.endpoints[0].trusted_output_keys.clear();
    assert_eq!(admit(&untrusted, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::UndeclaredOutputTrust));
    let mut ambiguous = selection(&ENDPOINT_NAMES);
    ambiguous.endpoints[0].trusted_output_keys.push(TrustedOutputKey {
        name: String::from("worker-a-1"),
        material_base64: String::from("other-material"),
    });
    assert_eq!(admit(&ambiguous, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::AmbiguousOutputKey));
}

#[test]
fn capacity_is_never_inferred_from_concurrency() {
    let mut inferred = selection(&ENDPOINT_NAMES);
    for endpoint in &mut inferred.endpoints {
        endpoint.declares_resource_inventory = false;
    }
    assert_eq!(admit(&inferred, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::InferredCapacity));
    inferred.session.max_concurrency = 1;
    assert!(admit(&inferred, &ENDPOINT_NAMES, &ENDPOINT_NAMES).is_ok(), "one job needs no capacity declaration");
}

#[test]
fn declaration_shape_and_session_bounds_fail_closed() {
    let mut unbounded = selection(&ENDPOINT_NAMES);
    unbounded.session.job_count_max = 0;
    assert_eq!(admit(&unbounded, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::UnboundedSessionPolicy));
    let mut transport = selection(&ENDPOINT_NAMES);
    transport.endpoints[0].transport = String::from("nix-daemon");
    assert_eq!(admit(&transport, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::UnsupportedTransport));
    let mut modes = selection(&ENDPOINT_NAMES);
    modes.endpoints[0].capabilities.sandbox_mode = String::from("permissive");
    assert_eq!(admit(&modes, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::UnsupportedSandboxMode));
    let mut unnamed = selection(&ENDPOINT_NAMES);
    unnamed.pool_id = String::new();
    assert_eq!(admit(&unnamed, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::InvalidPoolId));
    let empty = selection(&[]);
    assert_eq!(admit(&empty, &ENDPOINT_NAMES, &ENDPOINT_NAMES), Err(PoolAdmissionError::EmptyPool));
}
