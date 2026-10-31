//! Explicit pool selection projection and preflight admission.
//!
//! The operator names one configured pool and supplies caller-owned bearer
//! descriptors per endpoint. This module projects the evaluated configuration
//! and the operator's current bindings into the pure admission core, so an
//! invalid pool fails before any transport, upload, reservation, or build.

use std::collections::BTreeSet;
use std::path::Path;

use crunch_remote_core::endpoint::EndpointBindingTable;
use crunch_remote_core::pool::CallerCredential;
use crunch_remote_core::pool::DeclaredCapabilities;
use crunch_remote_core::pool::PoolAdmissionError;
use crunch_remote_core::pool::PoolEndpoint;
use crunch_remote_core::pool::PoolSelection;
use crunch_remote_core::pool::PoolSessionPolicy;
use crunch_remote_core::pool::TrustedOutputKey;
use crunch_remote_core::pool::admits_pool;

use crate::errors::RunError;
use crate::host_environment::unix_time_now_s;
use crate::remote_build::endpoint::EndpointBindingStore;
use crate::remote_build::endpoint_config::RemoteEndpointBindingConfig;
use crate::remote_farm_config::RemoteBuildFarmConfig;
use crate::remote_farm_config::RemoteBuilderPool;
use crate::remote_farm_config::RemoteNetworkMode;
use crate::remote_farm_config::RemoteSandboxMode;
use crate::remote_farm_config::RemoteTransportMode;

/// Maximum caller-supplied endpoint descriptors one preflight accepts.
pub const MAX_POOL_TICKET_FDS: usize = crunch_remote_core::pool::MAX_POOL_ENDPOINTS;
/// Maximum bytes of one transport executable this preflight will hash.
pub const MAX_TRANSPORT_PROGRAM_BYTES: u64 = 268_435_456;

/// Report schema for one admitted pool selection.
pub const POOL_PREFLIGHT_REPORT_SCHEMA: &str = "mantle-remote-pool-preflight-v1";

/// One caller-owned bearer descriptor named for exactly one endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolTicketFd {
    pub endpoint_name: String,
    pub fd: i32,
}

/// Parse repeatable `ENDPOINT=FD` operator values without reading any bearer.
pub fn parse_pool_ticket_fds(values: &[String]) -> Result<Vec<PoolTicketFd>, RunError> {
    if values.len() > MAX_POOL_TICKET_FDS {
        return Err(pool_error("pool-ticket-fd-limit"));
    }
    let mut parsed = Vec::with_capacity(values.len());
    let mut names = BTreeSet::new();
    for value in values {
        let (endpoint_name, fd_value) = value.split_once('=').ok_or_else(|| pool_error("pool-ticket-fd-malformed"))?;
        if endpoint_name.is_empty() || endpoint_name.contains(char::is_whitespace) {
            return Err(pool_error("pool-ticket-fd-endpoint-invalid"));
        }
        let fd = fd_value.parse::<i32>().map_err(|_| pool_error("pool-ticket-fd-number-invalid"))?;
        if fd <= 2 {
            return Err(pool_error("pool-ticket-fd-standard-stream"));
        }
        if !names.insert(endpoint_name) {
            return Err(pool_error("pool-ticket-fd-duplicate-endpoint"));
        }
        parsed.push(PoolTicketFd {
            endpoint_name: endpoint_name.to_string(),
            fd,
        });
    }
    debug_assert!(parsed.len() <= MAX_POOL_TICKET_FDS);
    debug_assert!(parsed.iter().all(|entry| entry.fd > 2));
    Ok(parsed)
}

/// Verify each descriptor is open in this process. The descriptor is never
/// read, duplicated, or transferred here.
pub fn verify_owned_descriptors(fds: &[PoolTicketFd]) -> Result<(), RunError> {
    for entry in fds {
        let link = format!("/proc/self/fd/{}", entry.fd);
        std::fs::read_link(Path::new(&link)).map_err(|_| pool_error("pool-ticket-fd-not-owned"))?;
    }
    debug_assert!(fds.len() <= MAX_POOL_TICKET_FDS);
    debug_assert!(fds.iter().all(|entry| !entry.endpoint_name.is_empty()));
    Ok(())
}

/// Project one configured pool and the operator's current bindings into the
/// pure selection. No bearer, transport, or capacity fact is invented here.
pub fn project_pool_selection(
    farm: &RemoteBuildFarmConfig,
    pool_id: &str,
    bindings: &RemoteEndpointBindingConfig,
) -> Result<PoolSelection, RunError> {
    let pool = farm.pools.iter().find(|pool| pool.pool_id == pool_id).ok_or_else(|| pool_error("pool-unknown"))?;
    let endpoints = pool
        .endpoints
        .iter()
        .map(|endpoint| project_endpoint(pool, endpoint, bindings))
        .collect::<Result<Vec<_>, RunError>>()?;
    let selection = PoolSelection {
        pool_id: pool.pool_id.clone(),
        endpoints,
        session: PoolSessionPolicy {
            max_concurrency: pool.session.max_concurrency,
            job_count_max: pool.session.job_count_max,
            session_bytes_max: pool.session.session_bytes_max,
        },
    };
    debug_assert!(!selection.pool_id.is_empty());
    debug_assert_eq!(selection.endpoints.len(), pool.endpoints.len());
    Ok(selection)
}

fn project_endpoint(
    pool: &RemoteBuilderPool,
    endpoint: &crate::remote_farm_config::RemoteEndpoint,
    bindings: &RemoteEndpointBindingConfig,
) -> Result<PoolEndpoint, RunError> {
    let binding = bindings
        .build_endpoint(&endpoint.endpoint_id)
        .ok_or_else(|| pool_error("pool-endpoint-binding-missing"))?;
    if endpoint.credential_reference.is_empty() {
        return Err(pool_error("pool-endpoint-credential-reference-missing"));
    }
    if endpoint.transport_program.is_empty() {
        return Err(pool_error("pool-endpoint-transport-program-missing"));
    }
    let keys = endpoint
        .trusted_output_keys
        .iter()
        .map(|key| TrustedOutputKey {
            name: key.public_key_name.clone(),
            material_base64: key.public_key_base64.clone(),
        })
        .collect::<Vec<_>>();
    let projected = PoolEndpoint {
        presented_binding: binding.endpoint_binding.clone(),
        transport: transport_name(endpoint.transport).to_string(),
        capabilities: DeclaredCapabilities {
            system: endpoint.profile.system.clone(),
            sandbox_mode: sandbox_name(endpoint.profile.sandbox_mode).to_string(),
            network_mode: network_name(endpoint.profile.network_mode).to_string(),
            features: endpoint.profile.features.clone(),
        },
        trusted_output_keys: keys,
        credential_reference: endpoint.credential_reference.clone(),
        declares_resource_inventory: endpoint.profile.resource_inventory.is_some(),
    };
    debug_assert_eq!(projected.presented_binding.spec.name, endpoint.endpoint_id);
    debug_assert!(!pool.pool_id.is_empty());
    Ok(projected)
}

/// Admit one selected pool: parse descriptors, verify ownership, resolve every
/// endpoint against the current binding table, and apply the pure decision.
pub fn admit_selected_pool(
    farm: &RemoteBuildFarmConfig,
    pool_id: &str,
    bindings: &RemoteEndpointBindingConfig,
    ticket_fds: &[String],
    now_unix_s: u64,
) -> Result<PoolPreflightReport, RunError> {
    let parsed = parse_pool_ticket_fds(ticket_fds)?;
    verify_owned_descriptors(&parsed)?;
    let pool = farm.pools.iter().find(|pool| pool.pool_id == pool_id).ok_or_else(|| pool_error("pool-unknown"))?;
    let known: BTreeSet<&str> = pool.endpoints.iter().map(|endpoint| endpoint.endpoint_id.as_str()).collect();
    if parsed.iter().any(|entry| !known.contains(entry.endpoint_name.as_str())) {
        return Err(pool_error("pool-ticket-fd-unknown-endpoint"));
    }
    let credentials = pool
        .endpoints
        .iter()
        .map(|endpoint| CallerCredential {
            endpoint_name: endpoint.endpoint_id.clone(),
            reference: endpoint.credential_reference.clone(),
        })
        .collect::<Vec<_>>();
    let selection = project_pool_selection(farm, pool_id, bindings)?;
    let records = EndpointBindingStore::open(&bindings.endpoint_state_directory)?.records()?;
    let table = EndpointBindingTable::from_records(records).map_err(|_| pool_error("pool-binding-table-invalid"))?;
    let admitted = admits_pool(&selection, &table, &credentials, now_unix_s).map_err(admission_error)?;
    let transport_programs = verify_transport_programs(pool)?;
    let report = PoolPreflightReport {
        schema: POOL_PREFLIGHT_REPORT_SCHEMA.to_string(),
        pool_id: admitted.pool_id().to_string(),
        endpoints: admitted.endpoint_names().to_vec(),
        session: admitted.session(),
        transport_programs,
    };
    debug_assert!(!report.endpoints.is_empty());
    debug_assert_eq!(report.pool_id, pool_id);
    Ok(report)
}

/// One selected endpoint's transport executable, remeasured before use.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VerifiedTransportProgram {
    pub endpoint_name: String,
    pub program: String,
    pub digest_blake3: String,
}

/// Bounded operator-facing preflight result. It proves admission only.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PoolPreflightReport {
    pub schema: String,
    pub pool_id: String,
    pub endpoints: Vec<String>,
    pub session: PoolSessionPolicy,
    pub transport_programs: Vec<VerifiedTransportProgram>,
}

fn transport_name(mode: RemoteTransportMode) -> &'static str {
    match mode {
        RemoteTransportMode::Stdio => "stdio",
        RemoteTransportMode::SshStdio => "ssh-stdio",
        RemoteTransportMode::P2p => "p2p",
    }
}

fn sandbox_name(mode: RemoteSandboxMode) -> &'static str {
    match mode {
        RemoteSandboxMode::Practical => "practical",
        RemoteSandboxMode::Strict => "strict",
    }
}

fn network_name(mode: RemoteNetworkMode) -> &'static str {
    match mode {
        RemoteNetworkMode::None => "none",
        RemoteNetworkMode::Local => "local",
        RemoteNetworkMode::Full => "full",
    }
}

/// Remeasure every selected endpoint's transport executable against its
/// declared BLAKE3 identity. Nothing is executed or resolved through PATH.
fn verify_transport_programs(pool: &RemoteBuilderPool) -> Result<Vec<VerifiedTransportProgram>, RunError> {
    let mut verified = Vec::with_capacity(pool.endpoints.len());
    for endpoint in &pool.endpoints {
        let path = Path::new(&endpoint.transport_program);
        let metadata = std::fs::metadata(path).map_err(|_| pool_error("pool-transport-program-unreadable"))?;
        if !metadata.is_file() || metadata.len() > MAX_TRANSPORT_PROGRAM_BYTES {
            return Err(pool_error("pool-transport-program-unreadable"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err(pool_error("pool-transport-program-not-executable"));
            }
        }
        let bytes = std::fs::read(path).map_err(|_| pool_error("pool-transport-program-unreadable"))?;
        let digest = blake3::hash(&bytes).to_hex().to_string();
        if digest != endpoint.transport_program_digest_blake3 {
            return Err(pool_error("pool-transport-program-drift"));
        }
        verified.push(VerifiedTransportProgram {
            endpoint_name: endpoint.endpoint_id.clone(),
            program: endpoint.transport_program.clone(),
            digest_blake3: digest,
        });
    }
    debug_assert_eq!(verified.len(), pool.endpoints.len());
    debug_assert!(verified.iter().all(|program| program.digest_blake3.len() == 64));
    Ok(verified)
}

fn admission_error(error: PoolAdmissionError) -> RunError {
    let code = match error {
        PoolAdmissionError::InvalidPoolId => "pool-id-invalid",
        PoolAdmissionError::EmptyPool => "pool-empty",
        PoolAdmissionError::EndpointCountExceeded => "pool-endpoint-limit",
        PoolAdmissionError::DuplicateEndpoint => "pool-endpoint-duplicate",
        PoolAdmissionError::UnsupportedTransport => "pool-transport-unsupported",
        PoolAdmissionError::UndeclaredCapabilities => "pool-capabilities-undeclared",
        PoolAdmissionError::UnsupportedSandboxMode => "pool-sandbox-mode-unsupported",
        PoolAdmissionError::UnsupportedNetworkMode => "pool-network-mode-unsupported",
        PoolAdmissionError::CapabilityCountExceeded => "pool-capability-limit",
        PoolAdmissionError::UndeclaredOutputTrust => "pool-output-trust-undeclared",
        PoolAdmissionError::AmbiguousOutputKey => "pool-output-key-ambiguous",
        PoolAdmissionError::InvalidCredentialReference => "pool-credential-reference-invalid",
        PoolAdmissionError::DuplicateCredentialReference => "pool-credential-reference-duplicate",
        PoolAdmissionError::MissingCallerCredential => "pool-credential-missing",
        PoolAdmissionError::UnboundedSessionPolicy => "pool-session-unbounded",
        PoolAdmissionError::InferredCapacity => "pool-capacity-inferred",
        PoolAdmissionError::EndpointBinding(_) => "pool-endpoint-binding-not-current",
    };
    pool_error(code)
}

fn pool_error(code: &str) -> RunError {
    RunError::Internal(format!("remote-pool:{code}"))
}

/// Run one `remote pool` preflight action and print its bounded report.
pub fn cmd_remote_pool(action: crate::RemotePoolAction, json_output: bool) -> Result<(), RunError> {
    match action {
        crate::RemotePoolAction::Check {
            builder_config,
            builder_pool,
            remote_binding_config,
            builder_ticket_fds,
        } => {
            let farm = crate::remote_farm_config::load_remote_build_farm_config(&builder_config)
                .map_err(|reason| pool_error(&format!("pool-config-invalid:{reason}")))?;
            let bindings = RemoteEndpointBindingConfig::load(&remote_binding_config)?;
            let report = admit_selected_pool(&farm, &builder_pool, &bindings, &builder_ticket_fds, unix_time_now_s()?)?;
            print_pool_report(&report, json_output)?;
            Ok(())
        }
    }
}

fn print_pool_report(report: &PoolPreflightReport, json_output: bool) -> Result<(), RunError> {
    let rendered = if json_output {
        serde_json::to_string(report)
    } else {
        serde_json::to_string_pretty(report)
    }
    .map_err(|error| pool_error(&format!("pool-report-render:{error}")))?;
    println!("{rendered}");
    debug_assert!(!report.endpoints.is_empty());
    debug_assert_eq!(report.schema, POOL_PREFLIGHT_REPORT_SCHEMA);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::fd::AsRawFd as _;

    use crunch_remote_core::endpoint::EndpointBindingSpec;
    use crunch_remote_core::endpoint::EndpointKind;

    use super::*;
    use crate::remote_build::endpoint_config::BuildEndpointBindingConfig;
    use crate::remote_farm_config::RemoteBuilderPool;
    use crate::remote_farm_config::RemoteCapabilityProfile;
    use crate::remote_farm_config::RemoteEndpoint;
    use crate::remote_farm_config::RemotePoolSessionPolicy;
    use crate::remote_farm_config::TrustRoot;

    const NOW_UNIX_S: u64 = 1_000;
    const EXPIRES_UNIX_S: u64 = 10_000;
    const ENDPOINT_NAMES: [&str; 2] = ["worker-a", "worker-b"];

    fn spec(name: &str) -> EndpointBindingSpec {
        EndpointBindingSpec {
            kind: EndpointKind::BuilderTicket,
            name: name.to_string(),
            scope: "native-build".to_string(),
            trust_requirement: "signed-content".to_string(),
            store_prefix: "/mantle/store".to_string(),
            expires_unix_s: EXPIRES_UNIX_S,
        }
    }

    fn build_config(name: &str) -> BuildEndpointBindingConfig {
        BuildEndpointBindingConfig {
            endpoint_binding: crunch_remote_core::endpoint::EndpointBinding {
                spec: spec(name),
                generation: 1,
            },
            scope: "native-build".to_string(),
            trust_requirement: "signed-content".to_string(),
        }
    }

    fn transport_program(dir: &Path) -> (std::path::PathBuf, String) {
        use std::os::unix::fs::PermissionsExt as _;
        let path = dir.join("transport.sh");
        std::fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("fixture transport program writes");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("fixture mode");
        let digest = blake3::hash(&std::fs::read(&path).expect("fixture read")).to_hex().to_string();
        (path, digest)
    }

    fn endpoint(name: &str, program: &Path, digest: &str) -> RemoteEndpoint {
        RemoteEndpoint {
            endpoint_id: name.to_string(),
            transport: RemoteTransportMode::Stdio,
            profile: RemoteCapabilityProfile {
                system: "x86_64-linux".to_string(),
                sandbox_mode: RemoteSandboxMode::Practical,
                network_mode: RemoteNetworkMode::None,
                features: vec!["native-compile".to_string()],
                worker_generation: 1,
                max_concurrency: 1,
                resource_inventory: None,
                max_upload_bytes: 1_073_741_824,
                max_build_time_secs: 3_600,
                retry_policy: crunch_remote_core::attempt::RemoteAttemptRetryPolicy::default(),
                transfer_policy: crunch_build::distributed::RemoteTransferPolicy::default(),
                workspace_policy: crate::remote_farm_config::RemoteWorkspacePolicy::default(),
            },
            credential_reference: format!("bearer-{name}"),
            trusted_output_keys: vec![TrustRoot {
                public_key_name: format!("{name}-1"),
                public_key_base64: format!("material-{name}"),
            }],
            transport_program: program.display().to_string(),
            transport_program_digest_blake3: digest.to_string(),
            transport_args: vec![format!("{name}.example")],
        }
    }

    fn farm(program: &Path, digest: &str) -> RemoteBuildFarmConfig {
        RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "native-pool".to_string(),
                endpoints: ENDPOINT_NAMES.iter().map(|name| endpoint(name, program, digest)).collect(),
                fallback_policy: crate::remote_farm_config::RemoteFallbackPolicy::TrustedOnly,
                output_trust_roots: Vec::new(),
                publishers: Vec::new(),
                session: RemotePoolSessionPolicy {
                    max_concurrency: 1,
                    job_count_max: 64,
                    session_bytes_max: 1_048_576,
                },
            }],
            ..RemoteBuildFarmConfig::default()
        }
    }

    fn seeded_bindings(dir: &Path) -> RemoteEndpointBindingConfig {
        let store = EndpointBindingStore::open(dir).expect("binding store opens");
        for name in ENDPOINT_NAMES {
            store.create(spec(name), None, NOW_UNIX_S).expect("fixture binding creates");
        }
        let builds = ENDPOINT_NAMES.iter().map(|name| build_config(name)).collect();
        RemoteEndpointBindingConfig::from_build_endpoints(dir.to_path_buf(), builds).expect("pool binding config")
    }

    fn ticket_values(fd: i32) -> Vec<String> {
        vec![format!("worker-a={fd}"), format!("worker-b={fd}")]
    }

    #[test]
    fn descriptor_arguments_are_parsed_and_ownership_is_checked() {
        let file = tempfile::tempfile().expect("temp descriptor");
        let fd = file.as_raw_fd();
        let parsed = parse_pool_ticket_fds(&ticket_values(fd)).expect("valid descriptors parse");
        assert_eq!(parsed.len(), 2);
        verify_owned_descriptors(&parsed).expect("open descriptors are owned");
        let missing = parse_pool_ticket_fds(&[String::from("worker-a=999999")]).expect("numeric descriptor parses");
        assert!(verify_owned_descriptors(&missing).is_err(), "closed descriptor is not owned");
        assert!(parse_pool_ticket_fds(&[String::from("worker-a=1")]).is_err(), "stdout is not a bearer");
        assert!(parse_pool_ticket_fds(&[String::from("worker-a=9"), String::from("worker-a=9")]).is_err());
        assert!(parse_pool_ticket_fds(&[String::from("worker-a")]).is_err());
    }

    #[test]
    fn projection_requires_a_configured_pool_binding_and_reference() {
        let dir = tempfile::tempdir().expect("binding state dir");
        let bindings = seeded_bindings(dir.path());
        let (program, digest) = transport_program(dir.path());
        let selection =
            project_pool_selection(&farm(&program, &digest), "native-pool", &bindings).expect("pool projects");
        assert_eq!(selection.endpoints.len(), 2);
        assert_eq!(selection.session.max_concurrency, 1);
        assert_eq!(selection.endpoints[0].credential_reference, "bearer-worker-a");
        assert!(project_pool_selection(&farm(&program, &digest), "missing-pool", &bindings).is_err());

        let partial =
            RemoteEndpointBindingConfig::from_build_endpoints(dir.path().to_path_buf(), vec![build_config("worker-a")])
                .expect("partial pool binding config");
        let error = project_pool_selection(&farm(&program, &digest), "native-pool", &partial)
            .expect_err("missing binding denies");
        assert!(error.to_string().contains("pool-endpoint-binding-missing"));

        let mut no_command = farm(&program, &digest);
        no_command.pools[0].endpoints[0].transport_program = String::new();
        let error = project_pool_selection(&no_command, "native-pool", &bindings)
            .expect_err("selected endpoint without a transport command denies");
        assert!(error.to_string().contains("pool-endpoint-transport-program-missing"));
    }

    #[test]
    fn preflight_admits_current_bindings_and_denies_revoked_ones() {
        let dir = tempfile::tempdir().expect("binding state dir");
        let bindings = seeded_bindings(dir.path());
        let (program, digest) = transport_program(dir.path());
        let farm = farm(&program, &digest);
        let file = tempfile::tempfile().expect("temp descriptor");
        let report = admit_selected_pool(&farm, "native-pool", &bindings, &ticket_values(file.as_raw_fd()), NOW_UNIX_S)
            .expect("pool admits");
        assert_eq!(report.pool_id, "native-pool");
        assert_eq!(report.endpoints, vec![String::from("worker-a"), String::from("worker-b")]);
        assert_eq!(report.schema, POOL_PREFLIGHT_REPORT_SCHEMA);
        assert_eq!(report.transport_programs.len(), 2);
        assert_eq!(report.transport_programs[0].digest_blake3, digest);

        let unknown = admit_selected_pool(
            &farm,
            "native-pool",
            &bindings,
            &[format!("worker-z={}", file.as_raw_fd())],
            NOW_UNIX_S,
        );
        assert!(unknown.is_err(), "an unselected endpoint descriptor is not authority");

        let mut concurrent = farm.clone();
        concurrent.pools[0].session.max_concurrency = 2;
        let inferred =
            admit_selected_pool(&concurrent, "native-pool", &bindings, &ticket_values(file.as_raw_fd()), NOW_UNIX_S);
        let error = inferred.expect_err("concurrency without declared inventory is not capacity");
        assert!(error.to_string().contains("pool-capacity-inferred"));

        let store = EndpointBindingStore::open(dir.path()).expect("binding store reopens");
        let presented =
            store.resolve(&build_config("worker-a").endpoint_binding, NOW_UNIX_S).expect("binding resolves");
        store.revoke(&presented).expect("fixture revocation");
        let denied = admit_selected_pool(&farm, "native-pool", &bindings, &ticket_values(file.as_raw_fd()), NOW_UNIX_S);
        let error = denied.expect_err("revoked binding denies the pool");
        assert!(error.to_string().contains("pool-endpoint-binding-not-current"));
    }

    #[test]
    fn transport_program_identity_is_remeasured_and_must_be_executable() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().expect("binding state dir");
        let bindings = seeded_bindings(dir.path());
        let (program, digest) = transport_program(dir.path());
        let file = tempfile::tempfile().expect("temp descriptor");
        std::fs::write(&program, b"#!/bin/sh\nexit 1\n").expect("program rewrite");
        let error = admit_selected_pool(
            &farm(&program, &digest),
            "native-pool",
            &bindings,
            &ticket_values(file.as_raw_fd()),
            NOW_UNIX_S,
        )
        .expect_err("changed executable denies the pool");
        assert!(error.to_string().contains("pool-transport-program-drift"));

        let (program, digest) = transport_program(dir.path());
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o644)).expect("fixture mode");
        let error = admit_selected_pool(
            &farm(&program, &digest),
            "native-pool",
            &bindings,
            &ticket_values(file.as_raw_fd()),
            NOW_UNIX_S,
        )
        .expect_err("non-executable transport denies the pool");
        assert!(error.to_string().contains("pool-transport-program-not-executable"));
    }
}
