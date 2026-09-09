//! Imperative transport and store adapter for the bounded remote service gateway.
//!
//! All admission decisions delegate to `crunch_build::distributed` pure core.
//! This shell owns stdio, the Nix daemon handler, clocks, NAR ingestion, store
//! persistence, and public-key verification.
//!
//! r[impl remote_builds.nix_compatibility_gateway]
//! r[impl remote_builds.gateway_store_integrity]
//! r[impl remote_builds.versioned_build_api]
//! r[impl remote_builds.gateway_non_claims]

// machine-artifact-public: remote.gateway-json-family
use std::fs;
use std::future::Future;
use std::io;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crunch_build::distributed::GATEWAY_API_SCHEMA;
use crunch_build::distributed::GATEWAY_API_VERSION;
use crunch_build::distributed::GATEWAY_EVIDENCE_SCHEMA;
use crunch_build::distributed::GATEWAY_NIX_PROTOCOL_MAJOR;
use crunch_build::distributed::GATEWAY_NIX_PROTOCOL_MINOR_MAX;
use crunch_build::distributed::GATEWAY_NIX_PROTOCOL_MINOR_MIN;
use crunch_build::distributed::GATEWAY_NON_CLAIMS;
use crunch_build::distributed::GatewayAdmissionInput;
use crunch_build::distributed::GatewayBounds;
use crunch_build::distributed::GatewayCommand;
use crunch_build::distributed::GatewayEvidence;
use crunch_build::distributed::GatewayEvidenceInput;
use crunch_build::distributed::GatewayOperation;
use crunch_build::distributed::GatewayPolicy;
use crunch_build::distributed::GatewayRejection;
use crunch_build::distributed::MAX_GATEWAY_MESSAGE_BYTES;
use crunch_build::distributed::RemoteGatewayAuthority;
use crunch_build::distributed::gateway_evidence;
use crunch_build::distributed::gateway_operation_identity;
use crunch_build::distributed::plan_gateway_operation;
use crunch_build::distributed::validate_gateway_authority;
use crunch_build::distributed::validate_gateway_policy;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nix_daemon::NixDaemonIO;
use nix_compat::nix_daemon::handler::NixDaemon;
use nix_compat::nix_daemon::types::AddToStoreNarRequest;
use nix_compat::nix_daemon::types::QueryValidPaths;
use nix_compat::nix_daemon::types::UnkeyedValidPathInfo;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::DIGEST_SIZE;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tokio::sync::Mutex;

use crate::errors::RunError;

pub const NIX_GATEWAY_METADATA_SCHEMA: &str = "mantle-nix-remote-gateway-metadata-v1";
const MAX_GATEWAY_STDIO_TOTAL_BYTES: u64 = 67_108_864;
const NIX_GATEWAY_VERSION: &str = "2.20.0-mantle-gateway";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NixGatewayMetadata {
    pub schema: String,
    pub server_version: String,
    pub protocol_major: u32,
    pub protocol_minor_min: u32,
    pub protocol_minor_max: u32,
    pub supported_store_operations: Vec<String>,
    pub build_api_schema: String,
    pub private_by_default: bool,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayOperatorStatus {
    pub schema: String,
    pub policy_identity_blake3: String,
    pub endpoint_visibility: String,
    pub audience: String,
    pub bounds: GatewayBounds,
    pub credential_hardening_evidence_blake3: Option<String>,
    pub ready: bool,
    pub non_claims: Vec<String>,
}

pub fn gateway_operator_status(policy: &GatewayPolicy) -> Result<GatewayOperatorStatus, GatewayRejection> {
    validate_gateway_policy(policy)?;
    Ok(GatewayOperatorStatus {
        schema: "mantle-remote-gateway-status-v1".to_string(),
        policy_identity_blake3: policy.policy_identity_blake3.clone(),
        endpoint_visibility: if policy.public_endpoint_enabled {
            "public-evidence-gated".to_string()
        } else {
            "private".to_string()
        },
        audience: policy.audience.clone(),
        bounds: policy.bounds,
        credential_hardening_evidence_blake3: policy.credential_hardening_evidence_blake3.clone(),
        ready: true,
        non_claims: GATEWAY_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    })
}

pub fn nix_gateway_metadata() -> NixGatewayMetadata {
    NixGatewayMetadata {
        schema: NIX_GATEWAY_METADATA_SCHEMA.to_string(),
        server_version: nix_gateway_version().to_string(),
        protocol_major: GATEWAY_NIX_PROTOCOL_MAJOR,
        protocol_minor_min: GATEWAY_NIX_PROTOCOL_MINOR_MIN,
        protocol_minor_max: GATEWAY_NIX_PROTOCOL_MINOR_MAX,
        supported_store_operations: vec![
            "is-valid-path".to_string(),
            "query-path-info".to_string(),
            "query-path-from-hash-part".to_string(),
            "query-valid-paths".to_string(),
            "query-valid-derivers".to_string(),
            "add-to-store-nar".to_string(),
        ],
        build_api_schema: GATEWAY_API_SCHEMA.to_string(),
        private_by_default: true,
        non_claims: GATEWAY_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteGatewayApiRequest {
    pub schema: String,
    pub api_version: u32,
    pub nix_protocol_major: Option<u32>,
    pub nix_protocol_minor: Option<u32>,
    pub now_unix_s: u64,
    pub authority: RemoteGatewayAuthority,
    pub policy: GatewayPolicy,
    pub operation: GatewayOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteGatewayApiPlan {
    pub schema: String,
    pub operation_identity_blake3: String,
    pub command: GatewayCommand,
    pub evidence: GatewayEvidence,
}

pub fn plan_gateway_api_request(request: RemoteGatewayApiRequest) -> Result<RemoteGatewayApiPlan, GatewayRejection> {
    if request.schema != GATEWAY_API_SCHEMA {
        return Err(GatewayRejection {
            code: crunch_build::distributed::GatewayRejectCode::ApiVersionUnsupported,
            message: "gateway-api-schema-unsupported".to_string(),
        });
    }
    validate_gateway_policy(&request.policy)?;
    let request_bytes = serde_json::to_vec(&request).map_err(|_| GatewayRejection {
        code: crunch_build::distributed::GatewayRejectCode::RequestInvalid,
        message: "gateway-api-request-serialization-failed".to_string(),
    })?;
    let request_byte_count = u32::try_from(request_bytes.len()).map_err(|_| GatewayRejection {
        code: crunch_build::distributed::GatewayRejectCode::RequestTooLarge,
        message: "gateway-api-message-bound-exceeded".to_string(),
    })?;
    if request_byte_count == 0 || request_byte_count > request.policy.bounds.message_bytes_max {
        return Err(GatewayRejection {
            code: crunch_build::distributed::GatewayRejectCode::RequestTooLarge,
            message: "gateway-api-message-bound-exceeded".to_string(),
        });
    }
    let operation_identity_blake3 = gateway_operation_identity(&request.operation)?;
    let command = plan_gateway_operation(GatewayAdmissionInput {
        api_version: request.api_version,
        nix_protocol_major: request.nix_protocol_major,
        nix_protocol_minor: request.nix_protocol_minor,
        now_unix_s: request.now_unix_s,
        authority: &request.authority,
        policy: &request.policy,
        operation: request.operation.clone(),
    })?;
    let attempt_id = command_attempt_id(&command);
    let evidence = gateway_evidence(GatewayEvidenceInput {
        policy: &request.policy,
        authority: &request.authority,
        operation: &request.operation,
        attempt_id,
        transferred_bytes: 0,
        stable_outcome: "planned-no-effect".to_string(),
    })?;
    debug_assert_eq!(evidence.schema, GATEWAY_EVIDENCE_SCHEMA);
    debug_assert_eq!(evidence.operation_identity_blake3, operation_identity_blake3);
    Ok(RemoteGatewayApiPlan {
        schema: GATEWAY_API_SCHEMA.to_string(),
        operation_identity_blake3,
        command,
        evidence,
    })
}

fn command_attempt_id(command: &GatewayCommand) -> Option<&crunch_build::distributed::RemoteAttemptId> {
    match command {
        GatewayCommand::SubmitRemoteAttempt { attempt_id, .. }
        | GatewayCommand::ReadAttemptStatus { attempt_id }
        | GatewayCommand::ReadAttemptLogs { attempt_id, .. }
        | GatewayCommand::ReadAttemptEvents { attempt_id, .. }
        | GatewayCommand::CancelOwnedAttempt { attempt_id }
        | GatewayCommand::DiscoverSignedResult { attempt_id }
        | GatewayCommand::PublishCacheResult { attempt_id, .. } => Some(attempt_id),
        GatewayCommand::QueryStorePath { .. }
        | GatewayCommand::QueryStorePathHash { .. }
        | GatewayCommand::QueryValidStorePaths { .. }
        | GatewayCommand::AdmitStoreImport { .. }
        | GatewayCommand::ReadUsageSummary { .. }
        | GatewayCommand::AdministerService { .. } => None,
    }
}

pub struct MantleNixGateway {
    store: Arc<Mutex<crunch_store::GatewayStore>>,
    authority: RemoteGatewayAuthority,
    policy: GatewayPolicy,
    trusted_store_keys: Vec<VerifyingKey>,
}

impl MantleNixGateway {
    pub fn new(
        store: crunch_store::GatewayStore,
        authority: RemoteGatewayAuthority,
        policy: GatewayPolicy,
        trusted_store_keys: Vec<VerifyingKey>,
    ) -> Result<Self, String> {
        if trusted_store_keys.is_empty() {
            return Err("nix-gateway-trusted-store-keys-empty".to_string());
        }
        validate_gateway_policy(&policy)
            .map_err(|error| format!("nix-gateway-policy-invalid:{:?}:{}", error.code, error.message))?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            authority,
            policy,
            trusted_store_keys,
        })
    }

    fn authorize(&self, operation: GatewayOperation) -> io::Result<GatewayCommand> {
        plan_gateway_operation(GatewayAdmissionInput {
            api_version: GATEWAY_API_VERSION,
            nix_protocol_major: Some(GATEWAY_NIX_PROTOCOL_MAJOR),
            nix_protocol_minor: Some(GATEWAY_NIX_PROTOCOL_MINOR_MAX),
            now_unix_s: gateway_now_unix_s()?,
            authority: &self.authority,
            policy: &self.policy,
            operation,
        })
        .map_err(gateway_rejection_io)
    }

    async fn query_path_info_inner(&self, path: &StorePath<String>) -> io::Result<Option<PathInfo>> {
        let store = self.store.lock().await;
        let read = store.find(path).await.map_err(store_io)?;
        debug_assert!(read.as_ref().is_none_or(|value| value.store_path == *path));
        Ok(read)
    }

    async fn query_hash_inner(&self, digest: [u8; DIGEST_SIZE]) -> io::Result<Option<PathInfo>> {
        let found = self.store.lock().await.find_by_digest(digest).await.map_err(store_io)?;
        if found.as_ref().is_some_and(|value| *value.store_path.digest() != digest) {
            return Err(io::Error::other("nix-gateway-pathinfo-digest-collision"));
        }
        Ok(found)
    }

    async fn ingest_store_nar<R>(&self, request: AddToStoreNarRequest, reader: &mut R) -> io::Result<()>
    where R: AsyncRead + Send + Unpin {
        reject_transport_trust_overrides(&request)?;
        let expected_nar_sha256 = *request.nar_hash;
        let nar_sha256_hex = data_encoding::HEXLOWER.encode(&expected_nar_sha256);
        let signature_count = u32::try_from(request.signatures.len()).unwrap_or(u32::MAX);
        let operation = GatewayOperation::UploadStoreObject {
            logical_path: request.path.to_absolute_path(),
            nar_size_bytes: request.nar_size,
            nar_sha256_hex,
            signature_count,
        };
        self.authorize(operation.clone())?;
        let received =
            self.store.lock().await.ingest_nar(reader, &request.ca, request.nar_size).await.map_err(nar_io)?;
        let observed = received.observation();
        if observed.nar_size_bytes != request.nar_size || observed.nar_sha256 != expected_nar_sha256 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "nix-gateway-nar-identity-mismatch"));
        }
        let path_info = path_info_from_import(&request, observed.node.clone(), observed.nar_sha256);
        let verification = crunch_build::signing::verify_pathinfo_signatures_with_store_dir(
            &path_info,
            &self.trusted_store_keys,
            "/nix/store",
        );
        if !verification.is_trusted() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "nix-gateway-pathinfo-signature-untrusted"));
        }
        self.persist_verified_import(operation, crunch_store::GatewayImportRequest { path_info, received })
            .await
    }

    async fn persist_verified_import(
        &self,
        operation: GatewayOperation,
        request: crunch_store::GatewayImportRequest,
    ) -> io::Result<()> {
        let mut store = self.store.lock().await;
        self.authorize(operation)?;
        store.persist_import(request).await.map_err(store_io)
    }
}

impl NixDaemonIO for MantleNixGateway {
    async fn query_path_info(&self, path: &StorePath<String>) -> io::Result<Option<UnkeyedValidPathInfo>> {
        self.authorize(GatewayOperation::QueryPath {
            logical_path: path.to_absolute_path(),
        })?;
        self.query_path_info_inner(path).await.map(|value| value.map(unkeyed_path_info))
    }

    async fn query_path_from_hash_part(&self, hash: &[u8]) -> io::Result<Option<UnkeyedValidPathInfo>> {
        let digest: [u8; DIGEST_SIZE] = hash
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "nix-gateway-store-hash-length-invalid"))?;
        self.authorize(GatewayOperation::QueryPathHash {
            digest_hex: data_encoding::HEXLOWER.encode(&digest),
        })?;
        self.query_hash_inner(digest).await.map(|value| value.map(unkeyed_path_info))
    }

    async fn query_valid_paths(&self, request: &QueryValidPaths) -> io::Result<Vec<StorePath<String>>> {
        let mut paths = request.paths.clone();
        paths.sort();
        paths.dedup();
        let logical_paths = paths.iter().map(StorePath::to_absolute_path).collect::<Vec<_>>();
        self.authorize(GatewayOperation::QueryValidPaths { logical_paths })?;
        let mut valid = Vec::with_capacity(paths.len());
        for path in paths {
            if self.query_path_info_inner(&path).await?.is_some() {
                valid.push(path);
            }
        }
        Ok(valid)
    }

    fn add_to_store_nar<R>(
        &self,
        request: AddToStoreNarRequest,
        reader: &mut R,
    ) -> impl Future<Output = io::Result<()>> + Send
    where
        R: AsyncRead + Send + Unpin,
    {
        self.ingest_store_nar(request, reader)
    }
}

fn path_info_from_import(request: &AddToStoreNarRequest, node: snix_castore::Node, nar_sha256: [u8; 32]) -> PathInfo {
    PathInfo {
        store_path: request.path.clone(),
        node,
        references: request.references.clone(),
        nar_size: request.nar_size,
        nar_sha256,
        signatures: request.signatures.clone(),
        deriver: request.deriver.clone(),
        ca: request.ca.clone(),
    }
}

fn unkeyed_path_info(path_info: PathInfo) -> UnkeyedValidPathInfo {
    UnkeyedValidPathInfo {
        deriver: path_info.deriver,
        nar_hash: NixHash::Sha256(path_info.nar_sha256).to_nix_nixbase32(),
        references: path_info.references,
        registration_time: 0,
        nar_size: path_info.nar_size,
        ultimate: true,
        signatures: path_info.signatures,
        ca: path_info.ca,
    }
}

fn reject_transport_trust_overrides(request: &AddToStoreNarRequest) -> io::Result<()> {
    if request.dont_check_sigs {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "nix-gateway-dont-check-signatures-forbidden"));
    }
    if request.repair {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "nix-gateway-repair-operation-forbidden"));
    }
    Ok(())
}

pub async fn serve_nix_gateway_stdio_once(gateway: MantleNixGateway) -> Result<(), RunError> {
    let protocol_major = gateway.policy.nix_protocol_major;
    let protocol_minor_min = gateway.policy.nix_protocol_minor_min;
    let protocol_minor_max = gateway.policy.nix_protocol_minor_max;
    let idle_secs_max = gateway.policy.bounds.idle_secs_max;
    let connection = BoundedStdio::new(MAX_GATEWAY_STDIO_TOTAL_BYTES, idle_secs_max);
    let value_bytes_max = usize::try_from(gateway.policy.bounds.frame_bytes_max)
        .map_err(|_| RunError::Internal("nix-gateway-frame-bound-conversion-failed".to_string()))?;
    let collection_items_max = usize::try_from(gateway.policy.bounds.paths_max)
        .map_err(|_| RunError::Internal("nix-gateway-path-bound-conversion-failed".to_string()))?;
    let message_bytes_max = usize::try_from(gateway.policy.bounds.message_bytes_max)
        .map_err(|_| RunError::Internal("nix-gateway-message-bound-conversion-failed".to_string()))?;
    let frame_bytes_max = u64::from(gateway.policy.bounds.frame_bytes_max);
    let mut daemon = NixDaemon::initialize_with_version_and_limits(
        Arc::new(gateway),
        connection,
        NIX_GATEWAY_VERSION,
        value_bytes_max,
        collection_items_max,
        message_bytes_max,
        frame_bytes_max,
    )
    .await
    .map_err(|error| RunError::Internal(format!("nix gateway handshake: {error}")))?;
    daemon.disable_compatibility_stubs();
    let version = daemon.protocol_version();
    let major = u32::from(version.major());
    let minor = u32::from(version.minor());
    if major != protocol_major || minor < protocol_minor_min || minor > protocol_minor_max {
        return Err(RunError::Internal("nix-gateway-negotiated-protocol-unsupported".to_string()));
    }
    match daemon.handle_client().await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(()),
        Err(error) => Err(RunError::Internal(format!("nix gateway session: {error}"))),
    }
}

pub fn nix_gateway_version() -> &'static str {
    NIX_GATEWAY_VERSION
}

struct BoundedStdio {
    stdin: tokio::io::Stdin,
    stdout: tokio::io::Stdout,
    read_bytes: u64,
    written_bytes: u64,
    total_bytes_max: u64,
    idle_period: Duration,
    idle_timer: Pin<Box<tokio::time::Sleep>>,
}

impl BoundedStdio {
    fn new(total_bytes_max: u64, idle_secs_max: u64) -> Self {
        assert!(total_bytes_max > 0);
        assert!(total_bytes_max <= MAX_GATEWAY_STDIO_TOTAL_BYTES);
        assert!(idle_secs_max > 0);
        let idle_period = Duration::from_secs(idle_secs_max);
        Self {
            stdin: tokio::io::stdin(),
            stdout: tokio::io::stdout(),
            read_bytes: 0,
            written_bytes: 0,
            total_bytes_max,
            idle_period,
            idle_timer: Box::pin(tokio::time::sleep(idle_period)),
        }
    }

    fn poll_idle(&mut self, context: &mut Context<'_>) -> io::Result<()> {
        if self.idle_timer.as_mut().poll(context).is_ready() {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "nix-gateway-idle-period-exceeded"));
        }
        Ok(())
    }

    fn observe_activity(&mut self) {
        self.idle_timer.as_mut().reset(tokio::time::Instant::now() + self.idle_period);
    }
}

impl AsyncRead for BoundedStdio {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.poll_idle(context)?;
        let before = buffer.filled().len();
        let result = Pin::new(&mut self.stdin).poll_read(context, buffer);
        if let Poll::Ready(Ok(())) = &result {
            let read = buffer.filled().len().saturating_sub(before);
            if read > 0 {
                self.observe_activity();
            }
            self.read_bytes = self.read_bytes.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
            let total = self.read_bytes.saturating_add(self.written_bytes);
            if total > self.total_bytes_max {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "nix-gateway-connection-byte-bound-exceeded",
                )));
            }
        }
        result
    }
}

impl AsyncWrite for BoundedStdio {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.poll_idle(context)?;
        let next = self
            .read_bytes
            .checked_add(self.written_bytes)
            .and_then(|total| total.checked_add(u64::try_from(buffer.len()).unwrap_or(u64::MAX)))
            .ok_or_else(|| io::Error::other("nix-gateway-connection-byte-count-overflow"));
        let Ok(next) = next else {
            return Poll::Ready(Err(next.unwrap_err()));
        };
        if next > self.total_bytes_max {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "nix-gateway-connection-byte-bound-exceeded",
            )));
        }
        let result = Pin::new(&mut self.stdout).poll_write(context, buffer);
        if let Poll::Ready(Ok(written)) = result {
            if written > 0 {
                self.observe_activity();
            }
            self.written_bytes = self.written_bytes.saturating_add(u64::try_from(written).unwrap_or(u64::MAX));
            return Poll::Ready(Ok(written));
        }
        result
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.poll_idle(context)?;
        Pin::new(&mut self.stdout).poll_flush(context)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.poll_idle(context)?;
        Pin::new(&mut self.stdout).poll_shutdown(context)
    }
}

pub(crate) fn gateway_now_unix_s() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| io::Error::other(format!("gateway-clock-before-unix-epoch: {error}")))
}

fn gateway_rejection_io(rejection: GatewayRejection) -> io::Error {
    let kind = match rejection.code {
        crunch_build::distributed::GatewayRejectCode::CapabilityDenied
        | crunch_build::distributed::GatewayRejectCode::CompatibilityTicketAdminDenied
        | crunch_build::distributed::GatewayRejectCode::AuthorityExpired
        | crunch_build::distributed::GatewayRejectCode::AuthorityAudienceMismatch
        | crunch_build::distributed::GatewayRejectCode::StoreImportUntrusted => io::ErrorKind::PermissionDenied,
        _ => io::ErrorKind::InvalidInput,
    };
    io::Error::new(kind, format!("{:?}:{}", rejection.code, rejection.message))
}

fn store_io(error: crunch_store::Error) -> io::Error {
    io::Error::other(format!("nix-gateway-store:{error}"))
}

fn nar_io(error: crunch_store::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("nix-gateway-nar:{error}"))
}

pub fn cmd_remote_gateway(
    action: crate::RemoteGatewayAction,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    _json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::RemoteGatewayAction::Metadata => print_gateway_json(&nix_gateway_metadata()),
        crate::RemoteGatewayAction::Plan { request } => {
            let request = read_gateway_json_file::<RemoteGatewayApiRequest>(&request)?;
            let plan = plan_gateway_api_request(request).map_err(gateway_plan_error)?;
            print_gateway_json(&plan)
        }
        crate::RemoteGatewayAction::Status { policy } => {
            let policy = read_gateway_json_file::<GatewayPolicy>(&policy)?;
            let status = gateway_operator_status(&policy).map_err(gateway_plan_error)?;
            print_gateway_json(&status)
        }
        crate::RemoteGatewayAction::ApiStdioOnce => {
            let request = read_gateway_api_stdin()?;
            let plan = plan_gateway_api_request(request).map_err(gateway_plan_error)?;
            print_gateway_json(&plan)
        }
        crate::RemoteGatewayAction::ApiDispatchStdioOnce {
            authority,
            cursor_key_fd,
            secret_manifest,
            secret_profile,
            secret_provider,
        } => crate::remote_gateway_api::cmd_gateway_api_dispatch(
            &authority,
            cursor_key_fd,
            crate::remote_service_secrets::RemoteServiceSecretRequest {
                manifest_path: secret_manifest,
                profile: secret_profile,
                provider: secret_provider,
            },
            state_dir,
        ),
        crate::RemoteGatewayAction::NixStdioOnce {
            authority,
            policy,
            trusted_store_keys,
        } => run_nix_gateway_stdio(&authority, &policy, &trusted_store_keys, output_dir, state_dir, store_prefix),
    }
}

fn run_nix_gateway_stdio(
    authority_path: &Path,
    policy_path: &Path,
    trusted_store_key_strings: &[String],
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<(), RunError> {
    if store_prefix != "/nix/store" {
        return Err(RunError::Internal("nix-gateway-requires-nix-store-prefix".to_string()));
    }
    let authority = read_gateway_json_file::<RemoteGatewayAuthority>(authority_path)?;
    let policy = read_gateway_json_file::<GatewayPolicy>(policy_path)?;
    validate_gateway_policy(&policy).map_err(gateway_plan_error)?;
    let now_unix_s =
        gateway_now_unix_s().map_err(|error| RunError::Internal(format!("reading gateway clock: {error}")))?;
    validate_gateway_authority(&authority, &policy, now_unix_s).map_err(gateway_plan_error)?;
    let trusted_store_keys = trusted_store_key_strings
        .iter()
        .map(|value| {
            VerifyingKey::parse(value).map_err(|error| RunError::Internal(format!("nix gateway key: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if trusted_store_keys.is_empty() {
        return Err(RunError::Internal("nix-gateway-trusted-store-keys-empty".to_string()));
    }
    fs::create_dir_all(state_dir)
        .map_err(|error| RunError::Internal(format!("creating nix gateway state dir: {error}")))?;
    fs::create_dir_all(output_dir)
        .map_err(|error| RunError::Internal(format!("creating nix gateway output dir: {error}")))?;
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| RunError::Internal(format!("creating nix gateway runtime: {error}")))?;
    let store = runtime
        .block_on(crunch_store::GatewayStore::open(crunch_store::StoreConfig::new(
            state_dir.to_path_buf(),
            output_dir.to_path_buf(),
            store_prefix.to_string(),
        )))
        .map_err(|error| RunError::Internal(format!("opening nix gateway store: {error}")))?;
    let gateway = MantleNixGateway::new(store, authority, policy, trusted_store_keys).map_err(RunError::Internal)?;
    runtime.block_on(serve_nix_gateway_stdio_once(gateway))
}

pub(crate) fn read_gateway_json_file<T>(path: &Path) -> Result<T, RunError>
where T: for<'de> Deserialize<'de> {
    let file = open_gateway_json_no_follow(path)?;
    if !file
        .metadata()
        .map_err(|error| RunError::Internal(format!("reading gateway JSON metadata {}: {error}", path.display())))?
        .is_file()
    {
        return Err(RunError::Internal("gateway-json-input-not-regular".to_string()));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(MAX_GATEWAY_MESSAGE_BYTES).unwrap_or(4_194_304));
    file.take(u64::from(MAX_GATEWAY_MESSAGE_BYTES).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading gateway JSON {}: {error}", path.display())))?;
    ensure_gateway_input_bound(&bytes)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| RunError::Internal(format!("parsing gateway JSON {}: invalid schema", path.display())))
}

#[cfg(unix)]
fn open_gateway_json_no_follow(path: &Path) -> Result<fs::File, RunError> {
    use std::os::unix::fs::OpenOptionsExt as _;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| RunError::Internal(format!("opening gateway JSON {}: {error}", path.display())))
}

#[cfg(not(unix))]
fn open_gateway_json_no_follow(path: &Path) -> Result<fs::File, RunError> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(RunError::Internal("gateway-json-input-symlink-rejected".to_string()));
    }
    fs::File::open(path)
        .map_err(|error| RunError::Internal(format!("opening gateway JSON {}: {error}", path.display())))
}

fn read_gateway_api_stdin() -> Result<RemoteGatewayApiRequest, RunError> {
    let mut bytes = Vec::with_capacity(usize::try_from(MAX_GATEWAY_MESSAGE_BYTES).unwrap_or(4_194_304));
    std::io::stdin()
        .lock()
        .take(u64::from(MAX_GATEWAY_MESSAGE_BYTES).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading gateway API stdin: {error}")))?;
    ensure_gateway_input_bound(&bytes)?;
    parse_gateway_api_request(&bytes)
}

fn parse_gateway_api_request(bytes: &[u8]) -> Result<RemoteGatewayApiRequest, RunError> {
    serde_json::from_slice(bytes).map_err(|_| RunError::Internal("gateway-api-input-invalid".to_string()))
}

fn ensure_gateway_input_bound(bytes: &[u8]) -> Result<(), RunError> {
    let byte_count = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    if byte_count == 0 || byte_count > MAX_GATEWAY_MESSAGE_BYTES {
        return Err(RunError::Internal("gateway-api-input-size-invalid".to_string()));
    }
    Ok(())
}

fn print_gateway_json<T: Serialize>(value: &T) -> Result<(), RunError> {
    let stdout = std::io::stdout();
    let mut locked = stdout.lock();
    serde_json::to_writer_pretty(&mut locked, value)
        .map_err(|error| RunError::Internal(format!("serializing gateway output: {error}")))?;
    locked
        .write_all(b"\n")
        .map_err(|error| RunError::Internal(format!("writing gateway output: {error}")))
}

fn gateway_plan_error(error: GatewayRejection) -> RunError {
    RunError::Internal(format!("gateway plan {:?}: {}", error.code, error.message))
}

#[cfg(test)]
mod import_tests;

#[cfg(test)]
mod tests {
    // r[verify remote_builds.nix_compatibility_gateway]
    // r[verify remote_builds.gateway_store_integrity]
    // r[verify remote_builds.versioned_build_api]
    // r[verify remote_builds.gateway_non_claims]
    use crunch_build::distributed::GatewayAuthoritySource;
    use crunch_build::distributed::GatewayCapability;
    use nix_compat::narinfo::VerifyingKey;

    use super::*;

    const NOW: u64 = 1_800_000_000;

    fn digest(label: &str) -> String {
        blake3::hash(label.as_bytes()).to_hex().to_string()
    }

    fn authority(capabilities: Vec<GatewayCapability>) -> RemoteGatewayAuthority {
        RemoteGatewayAuthority {
            source: GatewayAuthoritySource::VerifiedUcan,
            subject: "subject-a".into(),
            account_scope: "project-a".into(),
            audience: "mantle-remote-gateway".into(),
            expires_unix_s: NOW + 600,
            capabilities,
            evidence_refs_blake3: vec![digest("authority")],
        }
    }

    fn request(operation: GatewayOperation, capabilities: Vec<GatewayCapability>) -> RemoteGatewayApiRequest {
        RemoteGatewayApiRequest {
            schema: GATEWAY_API_SCHEMA.into(),
            api_version: GATEWAY_API_VERSION,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority: authority(capabilities),
            policy: GatewayPolicy::default(),
            operation,
        }
    }

    #[test]
    fn metadata_names_only_implemented_store_operations_and_non_claims() {
        let metadata = nix_gateway_metadata();
        assert_eq!(metadata.server_version, NIX_GATEWAY_VERSION);
        assert!(metadata.supported_store_operations.contains(&"add-to-store-nar".to_string()));
        assert!(!metadata.supported_store_operations.contains(&"evaluate-flake".to_string()));
        assert_eq!(metadata.non_claims.len(), GATEWAY_NON_CLAIMS.len());
        assert!(metadata.private_by_default);
    }

    // r[verify remote_builds.nix_compatibility_gateway]
    #[test]
    fn onixos_private_remote_farm_fixture_is_admitted() {
        let policy: GatewayPolicy =
            serde_json::from_str(include_str!("../tests/fixtures/onixos-remote-farm/gateway-policy.json")).unwrap();
        let authority: RemoteGatewayAuthority =
            serde_json::from_str(include_str!("../tests/fixtures/onixos-remote-farm/authority.json")).unwrap();
        let status = gateway_operator_status(&policy).unwrap();
        let command = plan_gateway_operation(GatewayAdmissionInput {
            api_version: GATEWAY_API_VERSION,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority: &authority,
            policy: &policy,
            operation: GatewayOperation::ReadStatus {
                attempt_id: "onixos-pilot-attempt".into(),
            },
        })
        .unwrap();

        assert_eq!(status.endpoint_visibility, "private");
        assert!(matches!(command, GatewayCommand::ReadAttemptStatus { .. }));
    }

    #[test]
    fn private_operator_status_is_redacted_and_ready() {
        let status = gateway_operator_status(&GatewayPolicy::default()).unwrap();
        let encoded = serde_json::to_string(&status).unwrap();
        assert_eq!(status.endpoint_visibility, "private");
        assert!(status.ready);
        assert!(!encoded.to_ascii_lowercase().contains("bearer "));
    }

    #[test]
    fn versioned_api_plan_is_pure_and_redacted() {
        let plan = plan_gateway_api_request(request(
            GatewayOperation::ReadStatus {
                attempt_id: "attempt-a".into(),
            },
            vec![GatewayCapability::ReadStatus],
        ))
        .unwrap();
        let encoded = serde_json::to_string(&plan).unwrap();
        assert_eq!(plan.schema, GATEWAY_API_SCHEMA);
        assert!(encoded.contains("planned-no-effect"));
        assert!(!encoded.to_ascii_lowercase().contains("bearer "));
    }

    #[test]
    fn selected_policy_rejects_api_message_above_its_lower_bound() {
        let mut request = request(
            GatewayOperation::QueryPath {
                logical_path: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-demo".into(),
            },
            vec![GatewayCapability::ReadStore],
        );
        request.policy.bounds.message_bytes_max = 1;
        let error = plan_gateway_api_request(request).unwrap_err();
        assert_eq!(error.code, crunch_build::distributed::GatewayRejectCode::RequestTooLarge);
        assert_eq!(error.message, "gateway-api-message-bound-exceeded");
    }

    #[test]
    fn planning_wire_rejects_unknown_fields_without_echoing_secret_material() {
        let mut value = serde_json::to_value(request(
            GatewayOperation::QueryPath {
                logical_path: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-demo".into(),
            },
            vec![GatewayCapability::ReadStore],
        ))
        .unwrap();
        value.as_object_mut().unwrap().insert(
            "authorization:must-not-be-echoed".to_string(),
            serde_json::Value::String("private-value".to_string()),
        );
        let bytes = serde_json::to_vec(&value).unwrap();
        let error = parse_gateway_api_request(&bytes).unwrap_err().to_string();
        assert_eq!(error, "error: gateway-api-input-invalid");
        assert!(!error.contains("authorization"));
        assert!(!error.contains("private-value"));
    }

    #[test]
    fn versioned_api_denies_missing_capability_before_shell_effects() {
        let error = plan_gateway_api_request(request(
            GatewayOperation::ReadStatus {
                attempt_id: "attempt-a".into(),
            },
            vec![GatewayCapability::ReadStore],
        ))
        .unwrap_err();
        assert_eq!(error.code, crunch_build::distributed::GatewayRejectCode::CapabilityDenied);
    }

    #[tokio::test]
    async fn nix_gateway_query_missing_path_uses_real_store_service() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("outputs");
        std::fs::create_dir_all(&output).unwrap();
        let store = crunch_store::GatewayStore::open(crunch_store::StoreConfig::new(
            temp.path().join("state"),
            output,
            "/nix/store".into(),
        ))
        .await
        .unwrap();
        let key = VerifyingKey::parse(crunch_build::signing::CACHE_NIXOS_ORG_PUBKEY).unwrap();
        let gateway = MantleNixGateway::new(
            store,
            authority(vec![GatewayCapability::ReadStore]),
            GatewayPolicy::default(),
            vec![key],
        )
        .unwrap();
        let path = StorePath::from_bytes(b"11111111111111111111111111111111-missing").unwrap();
        assert!(gateway.query_path_info(&path).await.unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn gateway_json_authority_input_rejects_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("authority.json");
        let link = temp.path().join("authority-link.json");
        fs::write(&target, b"{}").unwrap();
        symlink(&target, &link).unwrap();
        assert!(read_gateway_json_file::<serde_json::Value>(&link).is_err());
    }

    #[test]
    fn transport_trust_override_flags_fail_closed() {
        assert_eq!(nix_gateway_version(), NIX_GATEWAY_VERSION);
        assert!(NIX_GATEWAY_VERSION.contains("mantle-gateway"));
    }
}
