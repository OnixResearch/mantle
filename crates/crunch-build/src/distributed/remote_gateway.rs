//! Pure admission and translation core for Mantle's bounded remote service gateway.
//!
//! Transport, persistence, clocks, cryptographic key loading, stores, and the
//! scheduler stay in shell adapters. This module accepts explicit facts and
//! returns typed commands or stable rejections.
//!
//! r[impl remote_builds.gateway_functional_core]
//! r[impl remote_builds.granular_service_authority]
//! r[impl remote_builds.gateway_bounds_and_recovery]
//! r[impl remote_builds.idempotent_completion_events]
//! r[impl remote_builds.gateway_non_claims]

use serde::Deserialize;
use serde::Serialize;

use super::RemoteAttemptId;

pub const GATEWAY_API_SCHEMA: &str = "mantle-remote-build-api-v1";
pub const GATEWAY_API_VERSION: u32 = 1;
pub const GATEWAY_CURSOR_SCHEMA: &str = "mgc1";
pub const GATEWAY_COMPLETION_EVENT_SCHEMA: &str = "mantle-remote-completion-event-v1";
pub const GATEWAY_EVIDENCE_SCHEMA: &str = "mantle-remote-gateway-evidence-v1";
pub const GATEWAY_VALENCE_OBSERVATION_SCHEMA: &str = "mantle-valence-build-service-observation-v1";
pub const VALENCE_BUILD_SERVICE_PROFILE_ID: &str = "valence.build-service-evidence.v1";
pub const VALENCE_BUILD_SERVICE_RECORDED_ONLY_ROLE: &str = "RecordedOnly";
pub const VALENCE_BUILD_SERVICE_REQUIRED_NON_CLAIM: &str = "build-service linkage proves canonical identity and bounded observation linkage only; it does not prove build correctness, hermeticity, sandboxing, host isolation, identity beyond checked claims, authorization freshness, future resource sufficiency, accounting fairness, external status correctness, or release eligibility";
/// Final tasks-gate receipt preserved in the archived
/// `harden-remote-credential-boundary` validation evidence.
pub const GATEWAY_CREDENTIAL_HARDENING_EVIDENCE_BLAKE3: &str =
    "644a7002dc721e081f3f09e26cc96e313f0e23ae40074ad159b69f56ea654060";
pub const GATEWAY_NIX_PROTOCOL_MAJOR: u32 = 1;
pub const GATEWAY_NIX_PROTOCOL_MINOR_MIN: u32 = 23;
pub const GATEWAY_NIX_PROTOCOL_MINOR_MAX: u32 = 37;
pub const MAX_GATEWAY_CONNECTIONS: u32 = 1_024;
pub const MAX_GATEWAY_FRAME_BYTES: u32 = 1_048_576;
pub const MAX_GATEWAY_MESSAGE_BYTES: u32 = 4_194_304;
pub const MAX_GATEWAY_CONCURRENT_OPERATIONS: u32 = 1_024;
pub const MAX_GATEWAY_IDLE_SECS: u64 = 3_600;
pub const MAX_GATEWAY_CURSOR_BYTES: u32 = 512;
pub const MAX_GATEWAY_LOG_WINDOW_BYTES: u32 = 1_048_576;
pub const MAX_GATEWAY_EVENT_PAGE_ITEMS: u32 = 1_024;
pub const MAX_GATEWAY_PARTIAL_TRANSFER_BYTES: u64 = 1_073_741_824;
pub const MAX_GATEWAY_PATHS: u32 = 4_096;
pub const MAX_GATEWAY_TEXT_BYTES: u32 = 4_096;
pub const MAX_GATEWAY_CAPABILITIES: u32 = 16;
pub const MAX_GATEWAY_EVIDENCE_REFS: u32 = 32;
pub const MAX_GATEWAY_USAGE_ITEMS: u32 = 4_096;
const BLAKE3_HEX_LENGTH: usize = 64;
const NIX_STORE_DIGEST_HEX_LENGTH: usize = 40;
const CURSOR_PART_COUNT: usize = 5;
const COMPLETION_EVENT_DOMAIN: &[u8] = b"mantle-remote-completion-event-v1";
const ATTEMPT_ID_DOMAIN: &[u8] = b"mantle-remote-gateway-attempt-v1";
const OPERATION_IDENTITY_DOMAIN: &[u8] = b"mantle-remote-gateway-operation-v1";
const CURSOR_SCOPE_DOMAIN: &[u8] = b"mantle-remote-gateway-cursor-scope-v1";
const CURSOR_MAC_DOMAIN: &[u8] = b"mantle-remote-gateway-cursor-mac-v1";
const EVIDENCE_IDENTITY_DOMAIN: &[u8] = b"mantle-remote-gateway-evidence-v1";
const SECRET_MARKERS: [&str; 8] = [
    "authorization:",
    "bearer ",
    "client_secret",
    "password",
    "private-key",
    "private_key",
    "token=",
    "x-api-key",
];

pub const GATEWAY_NON_CLAIMS: [&str; 7] = [
    "tested operations do not prove arbitrary Nix compatibility",
    "gateway admission does not prove evaluator correctness",
    "gateway admission does not prove hermeticity or sandboxing",
    "transport success does not prove output or PathInfo correctness",
    "gateway evidence does not prove release eligibility",
    "store presence does not grant execution or publication authority",
    VALENCE_BUILD_SERVICE_REQUIRED_NON_CLAIM,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GatewayCapability {
    SubmitBuild,
    UploadInput,
    ReadStore,
    ReadStatus,
    ReadLogs,
    CancelOwnedAttempt,
    PublishCache,
    AdministerService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GatewayAuthoritySource {
    CompatibilityTicket,
    VerifiedUcan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteGatewayAuthority {
    pub source: GatewayAuthoritySource,
    pub subject: String,
    pub account_scope: String,
    pub audience: String,
    pub expires_unix_s: u64,
    pub capabilities: Vec<GatewayCapability>,
    pub evidence_refs_blake3: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayBounds {
    pub connections_max: u32,
    pub frame_bytes_max: u32,
    pub message_bytes_max: u32,
    pub concurrent_operations_max: u32,
    pub idle_secs_max: u64,
    pub cursor_bytes_max: u32,
    pub log_window_bytes_max: u32,
    pub event_page_items_max: u32,
    pub partial_transfer_bytes_max: u64,
    pub paths_max: u32,
}

impl Default for GatewayBounds {
    fn default() -> Self {
        Self {
            connections_max: MAX_GATEWAY_CONNECTIONS,
            frame_bytes_max: MAX_GATEWAY_FRAME_BYTES,
            message_bytes_max: MAX_GATEWAY_MESSAGE_BYTES,
            concurrent_operations_max: MAX_GATEWAY_CONCURRENT_OPERATIONS,
            idle_secs_max: MAX_GATEWAY_IDLE_SECS,
            cursor_bytes_max: MAX_GATEWAY_CURSOR_BYTES,
            log_window_bytes_max: MAX_GATEWAY_LOG_WINDOW_BYTES,
            event_page_items_max: MAX_GATEWAY_EVENT_PAGE_ITEMS,
            partial_transfer_bytes_max: MAX_GATEWAY_PARTIAL_TRANSFER_BYTES,
            paths_max: MAX_GATEWAY_PATHS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayPolicy {
    pub policy_identity_blake3: String,
    pub audience: String,
    pub api_version: u32,
    pub nix_protocol_major: u32,
    pub nix_protocol_minor_min: u32,
    pub nix_protocol_minor_max: u32,
    pub bounds: GatewayBounds,
    pub public_endpoint_enabled: bool,
    pub credential_hardening_evidence_blake3: Option<String>,
}

impl Default for GatewayPolicy {
    fn default() -> Self {
        Self {
            policy_identity_blake3: blake3::hash(b"mantle-private-gateway-default-v1").to_hex().to_string(),
            audience: "mantle-remote-gateway".to_string(),
            api_version: GATEWAY_API_VERSION,
            nix_protocol_major: GATEWAY_NIX_PROTOCOL_MAJOR,
            nix_protocol_minor_min: GATEWAY_NIX_PROTOCOL_MINOR_MIN,
            nix_protocol_minor_max: GATEWAY_NIX_PROTOCOL_MINOR_MAX,
            bounds: GatewayBounds::default(),
            public_endpoint_enabled: false,
            credential_hardening_evidence_blake3: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GatewayOperation {
    QueryPath {
        logical_path: String,
    },
    QueryPathHash {
        digest_hex: String,
    },
    QueryValidPaths {
        logical_paths: Vec<String>,
    },
    UploadStoreObject {
        logical_path: String,
        nar_size_bytes: u64,
        nar_sha256_hex: String,
        signature_count: u32,
    },
    SubmitBuild {
        request_identity_blake3: String,
        idempotency_key: String,
        concrete_derivation_path: String,
        input_paths: Vec<String>,
        expected_output_paths: Vec<String>,
    },
    ReadStatus {
        attempt_id: String,
    },
    ReadLogRange {
        attempt_id: String,
        cursor: Option<String>,
        byte_count: u32,
    },
    ReadEventPage {
        attempt_id: String,
        cursor: Option<String>,
        item_count: u32,
    },
    CancelAttempt {
        attempt_id: String,
        owner_subject: String,
    },
    DiscoverSignedResult {
        attempt_id: String,
    },
    UsageSummary {
        account_scope: String,
        item_count: u32,
    },
    PublishCacheResult {
        attempt_id: String,
        result_identity_blake3: String,
    },
    AdministerService {
        action: String,
    },
    Unsupported {
        operation_code: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GatewayCommand {
    QueryStorePath {
        logical_path: String,
    },
    QueryStorePathHash {
        digest_hex: String,
    },
    QueryValidStorePaths {
        logical_paths: Vec<String>,
    },
    AdmitStoreImport {
        logical_path: String,
        nar_size_bytes: u64,
        nar_sha256_hex: String,
    },
    SubmitRemoteAttempt {
        attempt_id: RemoteAttemptId,
        request_identity_blake3: String,
        idempotency_key: String,
        concrete_derivation_path: String,
    },
    ReadAttemptStatus {
        attempt_id: RemoteAttemptId,
    },
    ReadAttemptLogs {
        attempt_id: RemoteAttemptId,
        cursor: Option<String>,
        byte_count: u32,
    },
    ReadAttemptEvents {
        attempt_id: RemoteAttemptId,
        cursor: Option<String>,
        item_count: u32,
    },
    CancelOwnedAttempt {
        attempt_id: RemoteAttemptId,
    },
    DiscoverSignedResult {
        attempt_id: RemoteAttemptId,
    },
    ReadUsageSummary {
        account_scope: String,
        item_count: u32,
    },
    PublishCacheResult {
        attempt_id: RemoteAttemptId,
        result_identity_blake3: String,
    },
    AdministerService {
        action: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GatewayRejectCode {
    ApiVersionUnsupported,
    ProtocolVersionUnsupported,
    PolicyInvalid,
    PublicExposureBlocked,
    AuthorityInvalid,
    AuthorityExpired,
    AuthorityAudienceMismatch,
    CapabilityDenied,
    CompatibilityTicketAdminDenied,
    OperationUnsupported,
    RequestInvalid,
    RequestTooLarge,
    StoreImportUntrusted,
    IdempotencyConflict,
    AttemptUnknown,
    AttemptOwnerMismatch,
    CursorInvalid,
    CursorExpired,
    CursorScopeMismatch,
    CompletionEventInvalid,
    SensitiveEvidenceRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayRejection {
    pub code: GatewayRejectCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayAdmissionInput<'a> {
    pub api_version: u32,
    pub nix_protocol_major: Option<u32>,
    pub nix_protocol_minor: Option<u32>,
    pub now_unix_s: u64,
    pub authority: &'a RemoteGatewayAuthority,
    pub policy: &'a GatewayPolicy,
    pub operation: GatewayOperation,
}

// r[impl remote_builds.gateway_functional_core]
pub fn plan_gateway_operation(input: GatewayAdmissionInput<'_>) -> Result<GatewayCommand, GatewayRejection> {
    validate_gateway_policy(input.policy)?;
    validate_protocol(input.api_version, input.nix_protocol_major, input.nix_protocol_minor, input.policy)?;
    validate_gateway_authority(input.authority, input.policy, input.now_unix_s)?;
    let capability = required_capability(&input.operation)?;
    if !input.authority.capabilities.contains(&capability) {
        return Err(reject(GatewayRejectCode::CapabilityDenied, "gateway-required-capability-missing"));
    }
    if input.authority.source == GatewayAuthoritySource::CompatibilityTicket
        && capability == GatewayCapability::AdministerService
    {
        return Err(reject(
            GatewayRejectCode::CompatibilityTicketAdminDenied,
            "compatibility-ticket-cannot-administer-service",
        ));
    }
    translate_operation(input.operation, input.authority, input.policy)
}

pub fn validate_gateway_policy(policy: &GatewayPolicy) -> Result<(), GatewayRejection> {
    if !is_blake3_hex(&policy.policy_identity_blake3) || !bounded_text(&policy.audience) {
        return Err(reject(GatewayRejectCode::PolicyInvalid, "gateway-policy-identity-or-audience-invalid"));
    }
    if policy.api_version != GATEWAY_API_VERSION || policy.nix_protocol_major != GATEWAY_NIX_PROTOCOL_MAJOR {
        return Err(reject(GatewayRejectCode::PolicyInvalid, "gateway-policy-version-invalid"));
    }
    if policy.nix_protocol_minor_min < GATEWAY_NIX_PROTOCOL_MINOR_MIN
        || policy.nix_protocol_minor_max > GATEWAY_NIX_PROTOCOL_MINOR_MAX
        || policy.nix_protocol_minor_min > policy.nix_protocol_minor_max
    {
        return Err(reject(GatewayRejectCode::PolicyInvalid, "gateway-policy-protocol-range-invalid"));
    }
    validate_bounds(policy.bounds)?;
    if policy.public_endpoint_enabled {
        let Some(evidence) = policy.credential_hardening_evidence_blake3.as_deref() else {
            return Err(reject(GatewayRejectCode::PublicExposureBlocked, "gateway-public-credential-evidence-missing"));
        };
        if evidence != GATEWAY_CREDENTIAL_HARDENING_EVIDENCE_BLAKE3 {
            return Err(reject(
                GatewayRejectCode::PublicExposureBlocked,
                "gateway-public-credential-evidence-not-archived-receipt",
            ));
        }
    }
    debug_assert!(is_blake3_hex(&policy.policy_identity_blake3));
    debug_assert!(bounded_text(&policy.audience));
    Ok(())
}

fn validate_bounds(bounds: GatewayBounds) -> Result<(), GatewayRejection> {
    let fixed_u32 = [
        (bounds.connections_max, MAX_GATEWAY_CONNECTIONS),
        (bounds.frame_bytes_max, MAX_GATEWAY_FRAME_BYTES),
        (bounds.message_bytes_max, MAX_GATEWAY_MESSAGE_BYTES),
        (bounds.concurrent_operations_max, MAX_GATEWAY_CONCURRENT_OPERATIONS),
        (bounds.cursor_bytes_max, MAX_GATEWAY_CURSOR_BYTES),
        (bounds.log_window_bytes_max, MAX_GATEWAY_LOG_WINDOW_BYTES),
        (bounds.event_page_items_max, MAX_GATEWAY_EVENT_PAGE_ITEMS),
        (bounds.paths_max, MAX_GATEWAY_PATHS),
    ];
    if fixed_u32.iter().any(|(value, maximum)| *value == 0 || value > maximum) {
        return Err(reject(GatewayRejectCode::PolicyInvalid, "gateway-policy-u32-bound-invalid"));
    }
    if bounds.idle_secs_max == 0 || bounds.idle_secs_max > MAX_GATEWAY_IDLE_SECS {
        return Err(reject(GatewayRejectCode::PolicyInvalid, "gateway-policy-idle-bound-invalid"));
    }
    if bounds.partial_transfer_bytes_max == 0 || bounds.partial_transfer_bytes_max > MAX_GATEWAY_PARTIAL_TRANSFER_BYTES
    {
        return Err(reject(GatewayRejectCode::PolicyInvalid, "gateway-policy-transfer-bound-invalid"));
    }
    debug_assert!(fixed_u32.iter().all(|(value, maximum)| *value > 0 && value <= maximum));
    debug_assert!(bounds.idle_secs_max > 0 && bounds.idle_secs_max <= MAX_GATEWAY_IDLE_SECS);
    Ok(())
}

fn validate_protocol(
    api_version: u32,
    nix_major: Option<u32>,
    nix_minor: Option<u32>,
    policy: &GatewayPolicy,
) -> Result<(), GatewayRejection> {
    if api_version != policy.api_version {
        return Err(reject(GatewayRejectCode::ApiVersionUnsupported, "gateway-api-version-unsupported"));
    }
    match (nix_major, nix_minor) {
        (None, None) => Ok(()),
        (Some(major), Some(minor))
            if major == policy.nix_protocol_major
                && minor >= policy.nix_protocol_minor_min
                && minor <= policy.nix_protocol_minor_max =>
        {
            Ok(())
        }
        _ => Err(reject(GatewayRejectCode::ProtocolVersionUnsupported, "gateway-nix-protocol-version-unsupported")),
    }
}

pub fn validate_gateway_authority(
    authority: &RemoteGatewayAuthority,
    policy: &GatewayPolicy,
    now_unix_s: u64,
) -> Result<(), GatewayRejection> {
    if !bounded_text(&authority.subject) || !bounded_text(&authority.account_scope) {
        return Err(reject(GatewayRejectCode::AuthorityInvalid, "gateway-authority-subject-or-scope-invalid"));
    }
    if authority.audience != policy.audience {
        return Err(reject(GatewayRejectCode::AuthorityAudienceMismatch, "gateway-authority-audience-mismatch"));
    }
    if authority.expires_unix_s <= now_unix_s {
        return Err(reject(GatewayRejectCode::AuthorityExpired, "gateway-authority-expired"));
    }
    let capability_count = u32::try_from(authority.capabilities.len())
        .map_err(|_| reject(GatewayRejectCode::AuthorityInvalid, "gateway-authority-capability-count-invalid"))?;
    let evidence_count = u32::try_from(authority.evidence_refs_blake3.len())
        .map_err(|_| reject(GatewayRejectCode::AuthorityInvalid, "gateway-authority-evidence-count-invalid"))?;
    if capability_count == 0 || capability_count > MAX_GATEWAY_CAPABILITIES {
        return Err(reject(GatewayRejectCode::AuthorityInvalid, "gateway-authority-capability-count-invalid"));
    }
    if evidence_count == 0 || evidence_count > MAX_GATEWAY_EVIDENCE_REFS {
        return Err(reject(GatewayRejectCode::AuthorityInvalid, "gateway-authority-evidence-count-invalid"));
    }
    if !sorted_unique(&authority.capabilities)
        || authority.evidence_refs_blake3.iter().any(|value| !is_blake3_hex(value))
    {
        return Err(reject(GatewayRejectCode::AuthorityInvalid, "gateway-authority-capability-or-evidence-invalid"));
    }
    debug_assert_eq!(authority.audience, policy.audience);
    debug_assert!(authority.expires_unix_s > now_unix_s);
    Ok(())
}

fn required_capability(operation: &GatewayOperation) -> Result<GatewayCapability, GatewayRejection> {
    match operation {
        GatewayOperation::QueryPath { .. }
        | GatewayOperation::QueryPathHash { .. }
        | GatewayOperation::QueryValidPaths { .. } => Ok(GatewayCapability::ReadStore),
        GatewayOperation::UploadStoreObject { .. } => Ok(GatewayCapability::UploadInput),
        GatewayOperation::SubmitBuild { .. } => Ok(GatewayCapability::SubmitBuild),
        GatewayOperation::ReadStatus { .. }
        | GatewayOperation::ReadEventPage { .. }
        | GatewayOperation::DiscoverSignedResult { .. }
        | GatewayOperation::UsageSummary { .. } => Ok(GatewayCapability::ReadStatus),
        GatewayOperation::ReadLogRange { .. } => Ok(GatewayCapability::ReadLogs),
        GatewayOperation::CancelAttempt { .. } => Ok(GatewayCapability::CancelOwnedAttempt),
        GatewayOperation::PublishCacheResult { .. } => Ok(GatewayCapability::PublishCache),
        GatewayOperation::AdministerService { .. } => Ok(GatewayCapability::AdministerService),
        GatewayOperation::Unsupported { .. } => {
            Err(reject(GatewayRejectCode::OperationUnsupported, "gateway-operation-unsupported"))
        }
    }
}

fn translate_operation(
    operation: GatewayOperation,
    authority: &RemoteGatewayAuthority,
    policy: &GatewayPolicy,
) -> Result<GatewayCommand, GatewayRejection> {
    match operation {
        GatewayOperation::QueryPath { logical_path } => translate_query_path(logical_path),
        GatewayOperation::QueryPathHash { digest_hex } => {
            if !is_store_digest_hex(&digest_hex) {
                return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-store-path-hash-invalid"));
            }
            Ok(GatewayCommand::QueryStorePathHash { digest_hex })
        }
        GatewayOperation::QueryValidPaths { logical_paths } => translate_query_paths(logical_paths, policy),
        GatewayOperation::UploadStoreObject {
            logical_path,
            nar_size_bytes,
            nar_sha256_hex,
            signature_count,
        } => translate_upload(logical_path, nar_size_bytes, nar_sha256_hex, signature_count, policy),
        GatewayOperation::SubmitBuild {
            request_identity_blake3,
            idempotency_key,
            concrete_derivation_path,
            input_paths,
            expected_output_paths,
        } => translate_submit(GatewaySubmitTranslation {
            request_identity_blake3,
            idempotency_key,
            concrete_derivation_path,
            input_paths,
            expected_output_paths,
            authority,
            policy,
        }),
        GatewayOperation::ReadStatus { attempt_id } => Ok(GatewayCommand::ReadAttemptStatus {
            attempt_id: parse_attempt_id(attempt_id)?,
        }),
        GatewayOperation::ReadLogRange {
            attempt_id,
            cursor,
            byte_count,
        } => translate_log_range(attempt_id, cursor, byte_count, policy),
        GatewayOperation::ReadEventPage {
            attempt_id,
            cursor,
            item_count,
        } => translate_event_page(attempt_id, cursor, item_count, policy),
        GatewayOperation::CancelAttempt {
            attempt_id,
            owner_subject,
        } => translate_cancel(attempt_id, owner_subject, authority),
        GatewayOperation::DiscoverSignedResult { attempt_id } => Ok(GatewayCommand::DiscoverSignedResult {
            attempt_id: parse_attempt_id(attempt_id)?,
        }),
        GatewayOperation::UsageSummary {
            account_scope,
            item_count,
        } => translate_usage(account_scope, item_count, authority),
        GatewayOperation::PublishCacheResult {
            attempt_id,
            result_identity_blake3,
        } => translate_publish(attempt_id, result_identity_blake3),
        GatewayOperation::AdministerService { action } => {
            if !bounded_text(&action) || looks_sensitive(&action) {
                return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-administration-action-invalid"));
            }
            Ok(GatewayCommand::AdministerService { action })
        }
        GatewayOperation::Unsupported { .. } => {
            Err(reject(GatewayRejectCode::OperationUnsupported, "gateway-operation-unsupported"))
        }
    }
}

fn translate_query_path(logical_path: String) -> Result<GatewayCommand, GatewayRejection> {
    require_store_path(&logical_path)?;
    Ok(GatewayCommand::QueryStorePath { logical_path })
}

fn translate_query_paths(
    logical_paths: Vec<String>,
    policy: &GatewayPolicy,
) -> Result<GatewayCommand, GatewayRejection> {
    validate_path_set(&logical_paths, policy.bounds.paths_max)?;
    Ok(GatewayCommand::QueryValidStorePaths { logical_paths })
}

fn translate_upload(
    logical_path: String,
    nar_size_bytes: u64,
    nar_sha256_hex: String,
    signature_count: u32,
    policy: &GatewayPolicy,
) -> Result<GatewayCommand, GatewayRejection> {
    require_store_path(&logical_path)?;
    if nar_size_bytes == 0 || nar_size_bytes > policy.bounds.partial_transfer_bytes_max {
        return Err(reject(GatewayRejectCode::RequestTooLarge, "gateway-store-import-size-invalid"));
    }
    if !is_sha256_hex(&nar_sha256_hex) || signature_count == 0 {
        return Err(reject(GatewayRejectCode::StoreImportUntrusted, "gateway-store-import-trust-facts-invalid"));
    }
    Ok(GatewayCommand::AdmitStoreImport {
        logical_path,
        nar_size_bytes,
        nar_sha256_hex,
    })
}

struct GatewaySubmitTranslation<'a> {
    request_identity_blake3: String,
    idempotency_key: String,
    concrete_derivation_path: String,
    input_paths: Vec<String>,
    expected_output_paths: Vec<String>,
    authority: &'a RemoteGatewayAuthority,
    policy: &'a GatewayPolicy,
}

fn translate_submit(input: GatewaySubmitTranslation<'_>) -> Result<GatewayCommand, GatewayRejection> {
    if !is_blake3_hex(&input.request_identity_blake3) || !bounded_text(&input.idempotency_key) {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-build-identity-or-idempotency-invalid"));
    }
    require_store_path(&input.concrete_derivation_path)?;
    if !input.concrete_derivation_path.ends_with(".drv") {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-build-concrete-derivation-required"));
    }
    validate_path_set(&input.input_paths, input.policy.bounds.paths_max)?;
    validate_path_set(&input.expected_output_paths, input.policy.bounds.paths_max)?;
    debug_assert!(is_blake3_hex(&input.request_identity_blake3));
    debug_assert!(input.concrete_derivation_path.ends_with(".drv"));
    let attempt_id = derive_gateway_attempt_id(GatewayAttemptIdentityInput {
        subject: &input.authority.subject,
        account_scope: &input.authority.account_scope,
        idempotency_key: &input.idempotency_key,
        request_identity_blake3: &input.request_identity_blake3,
    })?;
    Ok(GatewayCommand::SubmitRemoteAttempt {
        attempt_id,
        request_identity_blake3: input.request_identity_blake3,
        idempotency_key: input.idempotency_key,
        concrete_derivation_path: input.concrete_derivation_path,
    })
}

fn translate_log_range(
    attempt_id: String,
    cursor: Option<String>,
    byte_count: u32,
    policy: &GatewayPolicy,
) -> Result<GatewayCommand, GatewayRejection> {
    if byte_count == 0 || byte_count > policy.bounds.log_window_bytes_max {
        return Err(reject(GatewayRejectCode::RequestTooLarge, "gateway-log-window-invalid"));
    }
    validate_cursor_size(cursor.as_deref(), policy.bounds.cursor_bytes_max)?;
    Ok(GatewayCommand::ReadAttemptLogs {
        attempt_id: parse_attempt_id(attempt_id)?,
        cursor,
        byte_count,
    })
}

fn translate_event_page(
    attempt_id: String,
    cursor: Option<String>,
    item_count: u32,
    policy: &GatewayPolicy,
) -> Result<GatewayCommand, GatewayRejection> {
    if item_count == 0 || item_count > policy.bounds.event_page_items_max {
        return Err(reject(GatewayRejectCode::RequestTooLarge, "gateway-event-page-invalid"));
    }
    validate_cursor_size(cursor.as_deref(), policy.bounds.cursor_bytes_max)?;
    Ok(GatewayCommand::ReadAttemptEvents {
        attempt_id: parse_attempt_id(attempt_id)?,
        cursor,
        item_count,
    })
}

fn translate_cancel(
    attempt_id: String,
    owner_subject: String,
    authority: &RemoteGatewayAuthority,
) -> Result<GatewayCommand, GatewayRejection> {
    if owner_subject != authority.subject {
        return Err(reject(GatewayRejectCode::AttemptOwnerMismatch, "gateway-cancel-owner-mismatch"));
    }
    Ok(GatewayCommand::CancelOwnedAttempt {
        attempt_id: parse_attempt_id(attempt_id)?,
    })
}

fn translate_usage(
    account_scope: String,
    item_count: u32,
    authority: &RemoteGatewayAuthority,
) -> Result<GatewayCommand, GatewayRejection> {
    if account_scope != authority.account_scope {
        return Err(reject(GatewayRejectCode::AuthorityInvalid, "gateway-usage-account-scope-mismatch"));
    }
    if item_count == 0 || item_count > MAX_GATEWAY_USAGE_ITEMS {
        return Err(reject(GatewayRejectCode::RequestTooLarge, "gateway-usage-item-count-invalid"));
    }
    Ok(GatewayCommand::ReadUsageSummary {
        account_scope,
        item_count,
    })
}

fn translate_publish(attempt_id: String, result_identity_blake3: String) -> Result<GatewayCommand, GatewayRejection> {
    if !is_blake3_hex(&result_identity_blake3) {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-cache-result-identity-invalid"));
    }
    Ok(GatewayCommand::PublishCacheResult {
        attempt_id: parse_attempt_id(attempt_id)?,
        result_identity_blake3,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingGatewaySubmission {
    pub attempt_id: RemoteAttemptId,
    pub subject: String,
    pub account_scope: String,
    pub idempotency_key: String,
    pub request_identity_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewaySubmissionDecision {
    Create { attempt_id: RemoteAttemptId },
    Recover { attempt_id: RemoteAttemptId },
}

// r[impl remote_builds.gateway_bounds_and_recovery]
pub fn decide_gateway_submission(
    command: &GatewayCommand,
    authority: &RemoteGatewayAuthority,
    existing: Option<&ExistingGatewaySubmission>,
) -> Result<GatewaySubmissionDecision, GatewayRejection> {
    let GatewayCommand::SubmitRemoteAttempt {
        attempt_id,
        request_identity_blake3,
        idempotency_key,
        ..
    } = command
    else {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-idempotency-command-not-submit"));
    };
    let Some(existing) = existing else {
        return Ok(GatewaySubmissionDecision::Create {
            attempt_id: attempt_id.clone(),
        });
    };
    debug_assert!(matches!(command, GatewayCommand::SubmitRemoteAttempt { .. }));
    debug_assert!(!attempt_id.as_str().is_empty());
    let is_exact = existing.attempt_id == *attempt_id
        && existing.subject == authority.subject
        && existing.account_scope == authority.account_scope
        && existing.idempotency_key == *idempotency_key
        && existing.request_identity_blake3 == *request_identity_blake3;
    if !is_exact {
        return Err(reject(GatewayRejectCode::IdempotencyConflict, "gateway-idempotency-conflict"));
    }
    Ok(GatewaySubmissionDecision::Recover {
        attempt_id: existing.attempt_id.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayReconnectFacts {
    pub attempt_id: RemoteAttemptId,
    pub owner_subject: String,
    pub account_scope: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayReconnectDecision {
    Allow { attempt_id: RemoteAttemptId },
}

pub fn decide_gateway_reconnect(
    authority: &RemoteGatewayAuthority,
    facts: &GatewayReconnectFacts,
) -> Result<GatewayReconnectDecision, GatewayRejection> {
    if !facts.exists {
        return Err(reject(GatewayRejectCode::AttemptUnknown, "gateway-reconnect-attempt-unknown"));
    }
    if facts.owner_subject != authority.subject || facts.account_scope != authority.account_scope {
        return Err(reject(GatewayRejectCode::AttemptOwnerMismatch, "gateway-reconnect-owner-or-scope-mismatch"));
    }
    if !authority.capabilities.contains(&GatewayCapability::ReadStatus) {
        return Err(reject(GatewayRejectCode::CapabilityDenied, "gateway-reconnect-status-capability-missing"));
    }
    Ok(GatewayReconnectDecision::Allow {
        attempt_id: facts.attempt_id.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayCursorScope<'a> {
    pub subject: &'a str,
    pub account_scope: &'a str,
    pub attempt_id: &'a RemoteAttemptId,
    pub stream: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayCursor {
    pub next_sequence: u64,
    pub expires_unix_s: u64,
}

#[derive(Debug)]
pub struct GatewayCursorSealInput<'a> {
    pub scope: GatewayCursorScope<'a>,
    pub next_sequence: u64,
    pub expires_unix_s: u64,
    pub key: &'a [u8; 32],
}

pub fn seal_gateway_cursor(input: GatewayCursorSealInput<'_>) -> Result<String, GatewayRejection> {
    if input.next_sequence == 0 || input.expires_unix_s == 0 {
        return Err(reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-sequence-or-expiry-invalid"));
    }
    let scope_digest = gateway_cursor_scope_digest(&input.scope)?;
    let material = format!("{GATEWAY_CURSOR_SCHEMA}.{}.{}.{}", input.next_sequence, input.expires_unix_s, scope_digest);
    let mac = gateway_cursor_mac(input.key, &material);
    let token = format!("{material}.{mac}");
    debug_assert!(token.starts_with(GATEWAY_CURSOR_SCHEMA));
    debug_assert!(!token.contains(input.scope.subject));
    Ok(token)
}

pub fn verify_gateway_cursor(
    token: &str,
    scope: GatewayCursorScope<'_>,
    now_unix_s: u64,
    key: &[u8; 32],
) -> Result<GatewayCursor, GatewayRejection> {
    let cursor_bytes_max = usize::try_from(MAX_GATEWAY_CURSOR_BYTES)
        .map_err(|_| reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-bound-invalid"))?;
    if token.len() > cursor_bytes_max {
        return Err(reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-too-large"));
    }
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != CURSOR_PART_COUNT || parts.first().copied() != Some(GATEWAY_CURSOR_SCHEMA) {
        return Err(reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-shape-invalid"));
    }
    let next_sequence = parts[1]
        .parse::<u64>()
        .map_err(|_| reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-sequence-invalid"))?;
    let expires_unix_s = parts[2]
        .parse::<u64>()
        .map_err(|_| reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-expiry-invalid"))?;
    let expected_scope = gateway_cursor_scope_digest(&scope)?;
    if parts[3] != expected_scope {
        return Err(reject(GatewayRejectCode::CursorScopeMismatch, "gateway-cursor-scope-mismatch"));
    }
    let material = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);
    if parts[4] != gateway_cursor_mac(key, &material) {
        return Err(reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-authentication-invalid"));
    }
    if next_sequence == 0 || expires_unix_s <= now_unix_s {
        return Err(reject(GatewayRejectCode::CursorExpired, "gateway-cursor-expired"));
    }
    debug_assert!(next_sequence > 0);
    debug_assert!(expires_unix_s > now_unix_s);
    Ok(GatewayCursor {
        next_sequence,
        expires_unix_s,
    })
}

fn gateway_cursor_scope_digest(scope: &GatewayCursorScope<'_>) -> Result<String, GatewayRejection> {
    if !bounded_text(scope.subject) || !bounded_text(scope.account_scope) || !bounded_text(scope.stream) {
        return Err(reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-scope-invalid"));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(CURSOR_SCOPE_DOMAIN);
    hash_text(&mut hasher, scope.subject)?;
    hash_text(&mut hasher, scope.account_scope)?;
    hash_text(&mut hasher, scope.attempt_id.as_str())?;
    hash_text(&mut hasher, scope.stream)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn gateway_cursor_mac(key: &[u8; 32], material: &str) -> String {
    let mut hasher = blake3::Hasher::new_keyed(key);
    hasher.update(CURSOR_MAC_DOMAIN);
    hasher.update(material.as_bytes());
    hasher.finalize().to_hex().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GatewayTerminalClass {
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayCompletionEvent {
    pub schema: String,
    pub event_identity_blake3: String,
    pub attempt_id: RemoteAttemptId,
    pub request_identity_blake3: String,
    pub terminal_class: GatewayTerminalClass,
    pub result_evidence_identity_blake3: Option<String>,
    pub policy_identity_blake3: String,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedGatewayCompletionEvent {
    pub event: GatewayCompletionEvent,
    pub producer_key_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayCompletionPlanInput {
    pub attempt_id: RemoteAttemptId,
    pub request_identity_blake3: String,
    pub terminal_class: GatewayTerminalClass,
    pub result_evidence_identity_blake3: Option<String>,
    pub policy_identity_blake3: String,
    pub sequence: u64,
}

// r[impl remote_builds.idempotent_completion_events]
pub fn plan_gateway_completion_event(
    input: GatewayCompletionPlanInput,
) -> Result<GatewayCompletionEvent, GatewayRejection> {
    if !is_blake3_hex(&input.request_identity_blake3)
        || !is_blake3_hex(&input.policy_identity_blake3)
        || input.sequence == 0
    {
        return Err(reject(GatewayRejectCode::CompletionEventInvalid, "gateway-completion-base-facts-invalid"));
    }
    if input.result_evidence_identity_blake3.as_deref().is_some_and(|value| !is_blake3_hex(value)) {
        return Err(reject(GatewayRejectCode::CompletionEventInvalid, "gateway-completion-result-evidence-invalid"));
    }
    debug_assert!(is_blake3_hex(&input.request_identity_blake3));
    debug_assert!(is_blake3_hex(&input.policy_identity_blake3));
    let identity = completion_event_identity(&input)?;
    Ok(GatewayCompletionEvent {
        schema: GATEWAY_COMPLETION_EVENT_SCHEMA.to_string(),
        event_identity_blake3: identity,
        attempt_id: input.attempt_id,
        request_identity_blake3: input.request_identity_blake3,
        terminal_class: input.terminal_class,
        result_evidence_identity_blake3: input.result_evidence_identity_blake3,
        policy_identity_blake3: input.policy_identity_blake3,
        sequence: input.sequence,
    })
}

pub fn bind_gateway_completion_signature(
    event: GatewayCompletionEvent,
    producer_key_id: String,
    signature: String,
) -> Result<SignedGatewayCompletionEvent, GatewayRejection> {
    if !bounded_text(&producer_key_id) || !bounded_text(&signature) || looks_sensitive(&producer_key_id) {
        return Err(reject(GatewayRejectCode::CompletionEventInvalid, "gateway-completion-signature-binding-invalid"));
    }
    Ok(SignedGatewayCompletionEvent {
        event,
        producer_key_id,
        signature,
    })
}

fn completion_event_identity(input: &GatewayCompletionPlanInput) -> Result<String, GatewayRejection> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(COMPLETION_EVENT_DOMAIN);
    hash_text(&mut hasher, input.attempt_id.as_str())?;
    hash_text(&mut hasher, &input.request_identity_blake3)?;
    hash_text(&mut hasher, terminal_class_label(input.terminal_class))?;
    hash_text(&mut hasher, input.result_evidence_identity_blake3.as_deref().unwrap_or("none"))?;
    hash_text(&mut hasher, &input.policy_identity_blake3)?;
    hasher.update(&input.sequence.to_le_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}

fn terminal_class_label(class: GatewayTerminalClass) -> &'static str {
    match class {
        GatewayTerminalClass::Completed => "completed",
        GatewayTerminalClass::Failed => "failed",
        GatewayTerminalClass::Cancelled => "cancelled",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayValenceObservation {
    pub schema: String,
    pub profile_id: String,
    pub source_evidence_identity_blake3: String,
    pub producer_policy_identity_blake3: String,
    pub authority_evidence_refs_blake3: Vec<String>,
    pub operation_class: String,
    pub attempt_id: Option<String>,
    pub transferred_bytes: u64,
    pub stable_outcome: String,
    pub role: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayEvidence {
    pub schema: String,
    pub evidence_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub authority_evidence_refs_blake3: Vec<String>,
    pub operation_identity_blake3: String,
    pub operation_class: String,
    pub attempt_id: Option<String>,
    pub transferred_bytes: u64,
    pub stable_outcome: String,
    pub valence_observation: GatewayValenceObservation,
    pub non_claims: Vec<String>,
}

#[derive(Debug)]
pub struct GatewayEvidenceInput<'a> {
    pub policy: &'a GatewayPolicy,
    pub authority: &'a RemoteGatewayAuthority,
    pub operation: &'a GatewayOperation,
    pub attempt_id: Option<&'a RemoteAttemptId>,
    pub transferred_bytes: u64,
    pub stable_outcome: String,
}

pub fn gateway_evidence(input: GatewayEvidenceInput<'_>) -> Result<GatewayEvidence, GatewayRejection> {
    validate_gateway_policy(input.policy)?;
    if !bounded_text(&input.stable_outcome) || looks_sensitive(&input.stable_outcome) {
        return Err(reject(GatewayRejectCode::SensitiveEvidenceRejected, "gateway-evidence-outcome-sensitive"));
    }
    let operation_identity_blake3 = gateway_operation_identity(input.operation)?;
    let operation_class = operation_class(input.operation).to_string();
    let attempt = input.attempt_id.map(|value| value.as_str().to_string());
    let identity = evidence_identity(EvidenceIdentityInput {
        policy_identity: &input.policy.policy_identity_blake3,
        authority_refs: &input.authority.evidence_refs_blake3,
        operation_identity: &operation_identity_blake3,
        attempt_id: attempt.as_deref(),
        transferred_bytes: input.transferred_bytes,
        outcome: &input.stable_outcome,
    })?;
    let non_claims = GATEWAY_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let valence_observation = GatewayValenceObservation {
        schema: GATEWAY_VALENCE_OBSERVATION_SCHEMA.to_string(),
        profile_id: VALENCE_BUILD_SERVICE_PROFILE_ID.to_string(),
        source_evidence_identity_blake3: identity.clone(),
        producer_policy_identity_blake3: input.policy.policy_identity_blake3.clone(),
        authority_evidence_refs_blake3: input.authority.evidence_refs_blake3.clone(),
        operation_class: operation_class.clone(),
        attempt_id: attempt.clone(),
        transferred_bytes: input.transferred_bytes,
        stable_outcome: input.stable_outcome.clone(),
        role: VALENCE_BUILD_SERVICE_RECORDED_ONLY_ROLE.to_string(),
        non_claims: vec![VALENCE_BUILD_SERVICE_REQUIRED_NON_CLAIM.to_string()],
    };
    let evidence = GatewayEvidence {
        schema: GATEWAY_EVIDENCE_SCHEMA.to_string(),
        evidence_identity_blake3: identity,
        policy_identity_blake3: input.policy.policy_identity_blake3.clone(),
        authority_evidence_refs_blake3: input.authority.evidence_refs_blake3.clone(),
        operation_identity_blake3,
        operation_class,
        attempt_id: attempt,
        transferred_bytes: input.transferred_bytes,
        stable_outcome: input.stable_outcome,
        valence_observation,
        non_claims,
    };
    let encoded = serde_json::to_string(&evidence)
        .map_err(|_| reject(GatewayRejectCode::SensitiveEvidenceRejected, "gateway-evidence-serialization-failed"))?;
    if looks_sensitive(&encoded) {
        return Err(reject(GatewayRejectCode::SensitiveEvidenceRejected, "gateway-evidence-sensitive-content"));
    }
    debug_assert_eq!(evidence.schema, GATEWAY_EVIDENCE_SCHEMA);
    debug_assert!(is_blake3_hex(&evidence.evidence_identity_blake3));
    Ok(evidence)
}

pub fn gateway_operation_identity(operation: &GatewayOperation) -> Result<String, GatewayRejection> {
    let bytes = serde_json::to_vec(operation)
        .map_err(|_| reject(GatewayRejectCode::RequestInvalid, "gateway-operation-serialization-failed"))?;
    let byte_count = u32::try_from(bytes.len())
        .map_err(|_| reject(GatewayRejectCode::RequestTooLarge, "gateway-operation-serialized-size-invalid"))?;
    if byte_count == 0 || byte_count > MAX_GATEWAY_MESSAGE_BYTES {
        return Err(reject(GatewayRejectCode::RequestTooLarge, "gateway-operation-serialized-size-invalid"));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(OPERATION_IDENTITY_DOMAIN);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

struct EvidenceIdentityInput<'a> {
    policy_identity: &'a str,
    authority_refs: &'a [String],
    operation_identity: &'a str,
    attempt_id: Option<&'a str>,
    transferred_bytes: u64,
    outcome: &'a str,
}

fn evidence_identity(input: EvidenceIdentityInput<'_>) -> Result<String, GatewayRejection> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EVIDENCE_IDENTITY_DOMAIN);
    hash_text(&mut hasher, input.policy_identity)?;
    for reference in input.authority_refs {
        hash_text(&mut hasher, reference)?;
    }
    hash_text(&mut hasher, input.operation_identity)?;
    hash_text(&mut hasher, input.attempt_id.unwrap_or("none"))?;
    hasher.update(&input.transferred_bytes.to_le_bytes());
    hash_text(&mut hasher, input.outcome)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn operation_class(operation: &GatewayOperation) -> &'static str {
    match operation {
        GatewayOperation::QueryPath { .. } => "query-path",
        GatewayOperation::QueryPathHash { .. } => "query-path-hash",
        GatewayOperation::QueryValidPaths { .. } => "query-valid-paths",
        GatewayOperation::UploadStoreObject { .. } => "upload-store-object",
        GatewayOperation::SubmitBuild { .. } => "submit-build",
        GatewayOperation::ReadStatus { .. } => "read-status",
        GatewayOperation::ReadLogRange { .. } => "read-log-range",
        GatewayOperation::ReadEventPage { .. } => "read-event-page",
        GatewayOperation::CancelAttempt { .. } => "cancel-attempt",
        GatewayOperation::DiscoverSignedResult { .. } => "discover-signed-result",
        GatewayOperation::UsageSummary { .. } => "usage-summary",
        GatewayOperation::PublishCacheResult { .. } => "publish-cache-result",
        GatewayOperation::AdministerService { .. } => "administer-service",
        GatewayOperation::Unsupported { .. } => "unsupported",
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GatewayAttemptIdentityInput<'a> {
    pub subject: &'a str,
    pub account_scope: &'a str,
    pub idempotency_key: &'a str,
    pub request_identity_blake3: &'a str,
}

pub fn derive_gateway_attempt_id(input: GatewayAttemptIdentityInput<'_>) -> Result<RemoteAttemptId, GatewayRejection> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ATTEMPT_ID_DOMAIN);
    hash_text(&mut hasher, input.subject)?;
    hash_text(&mut hasher, input.account_scope)?;
    hash_text(&mut hasher, input.idempotency_key)?;
    hash_text(&mut hasher, input.request_identity_blake3)?;
    RemoteAttemptId::new(format!("gateway-{}", hasher.finalize().to_hex()))
        .map_err(|_| reject(GatewayRejectCode::RequestInvalid, "gateway-attempt-identity-invalid"))
}

fn parse_attempt_id(value: String) -> Result<RemoteAttemptId, GatewayRejection> {
    RemoteAttemptId::new(value)
        .map_err(|_| reject(GatewayRejectCode::RequestInvalid, "gateway-attempt-identity-invalid"))
}

fn validate_path_set(paths: &[String], maximum: u32) -> Result<(), GatewayRejection> {
    let count = u32::try_from(paths.len())
        .map_err(|_| reject(GatewayRejectCode::RequestTooLarge, "gateway-path-count-invalid"))?;
    if count == 0 || count > maximum {
        return Err(reject(GatewayRejectCode::RequestTooLarge, "gateway-path-count-invalid"));
    }
    if !sorted_unique(paths) {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-paths-not-sorted-unique"));
    }
    for path in paths {
        require_store_path(path)?;
    }
    Ok(())
}

fn validate_cursor_size(cursor: Option<&str>, maximum: u32) -> Result<(), GatewayRejection> {
    let Some(cursor) = cursor else {
        return Ok(());
    };
    let length_bytes = u32::try_from(cursor.len())
        .map_err(|_| reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-size-invalid"))?;
    if length_bytes == 0 || length_bytes > maximum {
        return Err(reject(GatewayRejectCode::CursorInvalid, "gateway-cursor-size-invalid"));
    }
    Ok(())
}

fn require_store_path(path: &str) -> Result<(), GatewayRejection> {
    let is_valid_prefix = path.starts_with("/nix/store/") || path.starts_with("/mantle/store/");
    if !is_valid_prefix {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-store-path-invalid"));
    }
    if !bounded_text(path) || path.contains("..") {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-store-path-invalid"));
    }
    if path.chars().any(char::is_whitespace) {
        return Err(reject(GatewayRejectCode::RequestInvalid, "gateway-store-path-invalid"));
    }
    Ok(())
}

fn bounded_text(value: &str) -> bool {
    let Ok(length_bytes) = u32::try_from(value.len()) else {
        return false;
    };
    length_bytes > 0 && length_bytes <= MAX_GATEWAY_TEXT_BYTES && !value.chars().any(char::is_control)
}

fn sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_sha256_hex(value: &str) -> bool {
    is_blake3_hex(value)
}

fn is_store_digest_hex(value: &str) -> bool {
    value.len() == NIX_STORE_DIGEST_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn looks_sensitive(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    SECRET_MARKERS.iter().any(|marker| lower.contains(marker))
}

fn hash_text(hasher: &mut blake3::Hasher, value: &str) -> Result<(), GatewayRejection> {
    let length_bytes = u64::try_from(value.len())
        .map_err(|_| reject(GatewayRejectCode::RequestTooLarge, "gateway-text-length-overflow"))?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

fn reject(code: GatewayRejectCode, message: &str) -> GatewayRejection {
    debug_assert!(!message.is_empty());
    debug_assert!(!looks_sensitive(message));
    GatewayRejection {
        code,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    // r[verify remote_builds.gateway_functional_core]
    // r[verify remote_builds.granular_service_authority]
    // r[verify remote_builds.gateway_bounds_and_recovery]
    // r[verify remote_builds.idempotent_completion_events]
    // r[verify remote_builds.gateway_non_claims]
    use super::*;

    const NOW: u64 = 1_800_000_000;

    fn digest(label: &str) -> String {
        blake3::hash(label.as_bytes()).to_hex().to_string()
    }

    fn policy() -> GatewayPolicy {
        GatewayPolicy::default()
    }

    fn authority(source: GatewayAuthoritySource) -> RemoteGatewayAuthority {
        RemoteGatewayAuthority {
            source,
            subject: "subject-a".to_string(),
            account_scope: "project-a".to_string(),
            audience: "mantle-remote-gateway".to_string(),
            expires_unix_s: NOW + 600,
            capabilities: vec![
                GatewayCapability::SubmitBuild,
                GatewayCapability::UploadInput,
                GatewayCapability::ReadStore,
                GatewayCapability::ReadStatus,
                GatewayCapability::ReadLogs,
                GatewayCapability::CancelOwnedAttempt,
                GatewayCapability::PublishCache,
            ],
            evidence_refs_blake3: vec![digest("authority")],
        }
    }

    fn admit(
        operation: GatewayOperation,
        authority: &RemoteGatewayAuthority,
    ) -> Result<GatewayCommand, GatewayRejection> {
        plan_gateway_operation(GatewayAdmissionInput {
            api_version: GATEWAY_API_VERSION,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority,
            policy: &policy(),
            operation,
        })
    }

    fn build_operation() -> GatewayOperation {
        GatewayOperation::SubmitBuild {
            request_identity_blake3: digest("request"),
            idempotency_key: "ci-run-7".to_string(),
            concrete_derivation_path: "/nix/store/00000000000000000000000000000000-demo.drv".to_string(),
            input_paths: vec!["/nix/store/11111111111111111111111111111111-input".to_string()],
            expected_output_paths: vec!["/nix/store/22222222222222222222222222222222-output".to_string()],
        }
    }

    fn attempt() -> RemoteAttemptId {
        RemoteAttemptId::new("attempt-a").unwrap()
    }

    #[test]
    fn admitted_operation_table_translates_every_public_class() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let operations = vec![
            GatewayOperation::QueryPath {
                logical_path: "/nix/store/11111111111111111111111111111111-a".into(),
            },
            GatewayOperation::QueryPathHash {
                digest_hex: "11".repeat(20),
            },
            GatewayOperation::QueryValidPaths {
                logical_paths: vec!["/nix/store/11111111111111111111111111111111-a".into()],
            },
            GatewayOperation::UploadStoreObject {
                logical_path: "/nix/store/11111111111111111111111111111111-a".into(),
                nar_size_bytes: 1,
                nar_sha256_hex: digest("nar"),
                signature_count: 1,
            },
            build_operation(),
            GatewayOperation::ReadStatus {
                attempt_id: "attempt-a".into(),
            },
            GatewayOperation::ReadLogRange {
                attempt_id: "attempt-a".into(),
                cursor: None,
                byte_count: 1,
            },
            GatewayOperation::ReadEventPage {
                attempt_id: "attempt-a".into(),
                cursor: None,
                item_count: 1,
            },
            GatewayOperation::CancelAttempt {
                attempt_id: "attempt-a".into(),
                owner_subject: "subject-a".into(),
            },
            GatewayOperation::DiscoverSignedResult {
                attempt_id: "attempt-a".into(),
            },
            GatewayOperation::UsageSummary {
                account_scope: "project-a".into(),
                item_count: 1,
            },
            GatewayOperation::PublishCacheResult {
                attempt_id: "attempt-a".into(),
                result_identity_blake3: digest("result"),
            },
        ];
        for operation in operations {
            assert!(admit(operation, &auth).is_ok());
        }
    }

    #[test]
    fn unsupported_evaluator_registry_and_arbitrary_operations_fail_before_dispatch() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        for operation_code in [2_u64, 14, 47, 9_999] {
            let error = admit(GatewayOperation::Unsupported { operation_code }, &auth).unwrap_err();
            assert_eq!(error.code, GatewayRejectCode::OperationUnsupported);
        }
    }

    #[test]
    fn granular_capabilities_do_not_expand_read_authority() {
        let mut auth = authority(GatewayAuthoritySource::VerifiedUcan);
        auth.capabilities = vec![GatewayCapability::ReadStatus];
        let status = admit(
            GatewayOperation::ReadStatus {
                attempt_id: "attempt-a".into(),
            },
            &auth,
        );
        let build = admit(build_operation(), &auth);
        assert!(status.is_ok());
        assert_eq!(build.unwrap_err().code, GatewayRejectCode::CapabilityDenied);
    }

    #[test]
    fn compatibility_ticket_cannot_administer_even_when_claimed() {
        let mut auth = authority(GatewayAuthoritySource::CompatibilityTicket);
        auth.capabilities.push(GatewayCapability::AdministerService);
        auth.capabilities.sort();
        let error = admit(
            GatewayOperation::AdministerService {
                action: "rotate-service-keys".into(),
            },
            &auth,
        )
        .unwrap_err();
        assert_eq!(error.code, GatewayRejectCode::CompatibilityTicketAdminDenied);
    }

    #[test]
    fn verified_ucan_admin_requires_explicit_capability() {
        let mut auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let denied = admit(GatewayOperation::AdministerService { action: "drain".into() }, &auth).unwrap_err();
        auth.capabilities.push(GatewayCapability::AdministerService);
        auth.capabilities.sort();
        let allowed = admit(GatewayOperation::AdministerService { action: "drain".into() }, &auth);
        assert_eq!(denied.code, GatewayRejectCode::CapabilityDenied);
        assert!(allowed.is_ok());
    }

    #[test]
    fn expired_wrong_audience_and_unsorted_capabilities_fail_closed() {
        let mut auth = authority(GatewayAuthoritySource::VerifiedUcan);
        auth.expires_unix_s = NOW;
        assert_eq!(admit(build_operation(), &auth).unwrap_err().code, GatewayRejectCode::AuthorityExpired);
        auth.expires_unix_s = NOW + 1;
        auth.audience = "other".into();
        assert_eq!(admit(build_operation(), &auth).unwrap_err().code, GatewayRejectCode::AuthorityAudienceMismatch);
        auth.audience = "mantle-remote-gateway".into();
        auth.capabilities.reverse();
        assert_eq!(admit(build_operation(), &auth).unwrap_err().code, GatewayRejectCode::AuthorityInvalid);
    }

    #[test]
    fn public_exposure_requires_exact_archived_credential_evidence() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let mut public = policy();
        public.public_endpoint_enabled = true;
        let missing = plan_gateway_operation(GatewayAdmissionInput {
            api_version: 1,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority: &auth,
            policy: &public,
            operation: build_operation(),
        })
        .unwrap_err();
        public.credential_hardening_evidence_blake3 = Some(digest("credential-hardening"));
        let wrong = plan_gateway_operation(GatewayAdmissionInput {
            api_version: 1,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority: &auth,
            policy: &public,
            operation: build_operation(),
        })
        .unwrap_err();
        public.credential_hardening_evidence_blake3 = Some(GATEWAY_CREDENTIAL_HARDENING_EVIDENCE_BLAKE3.to_string());
        let admitted = plan_gateway_operation(GatewayAdmissionInput {
            api_version: 1,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority: &auth,
            policy: &public,
            operation: build_operation(),
        });
        assert_eq!(missing.code, GatewayRejectCode::PublicExposureBlocked);
        assert_eq!(wrong.code, GatewayRejectCode::PublicExposureBlocked);
        assert!(admitted.is_ok());
    }

    #[test]
    fn protocol_range_is_explicit_and_unknown_versions_fail() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let check = |minor| {
            plan_gateway_operation(GatewayAdmissionInput {
                api_version: 1,
                nix_protocol_major: Some(1),
                nix_protocol_minor: Some(minor),
                now_unix_s: NOW,
                authority: &auth,
                policy: &policy(),
                operation: GatewayOperation::QueryPath {
                    logical_path: "/nix/store/11111111111111111111111111111111-a".into(),
                },
            })
        };
        assert!(check(GATEWAY_NIX_PROTOCOL_MINOR_MIN).is_ok());
        assert!(check(GATEWAY_NIX_PROTOCOL_MINOR_MAX).is_ok());
        assert_eq!(check(20).unwrap_err().code, GatewayRejectCode::ProtocolVersionUnsupported);
        assert_eq!(check(38).unwrap_err().code, GatewayRejectCode::ProtocolVersionUnsupported);
    }

    #[test]
    fn every_named_bound_rejects_zero_or_oversized_values() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let oversized_log = admit(
            GatewayOperation::ReadLogRange {
                attempt_id: "attempt-a".into(),
                cursor: None,
                byte_count: MAX_GATEWAY_LOG_WINDOW_BYTES + 1,
            },
            &auth,
        );
        let oversized_events = admit(
            GatewayOperation::ReadEventPage {
                attempt_id: "attempt-a".into(),
                cursor: None,
                item_count: MAX_GATEWAY_EVENT_PAGE_ITEMS + 1,
            },
            &auth,
        );
        let oversized_upload = admit(
            GatewayOperation::UploadStoreObject {
                logical_path: "/nix/store/11111111111111111111111111111111-a".into(),
                nar_size_bytes: MAX_GATEWAY_PARTIAL_TRANSFER_BYTES + 1,
                nar_sha256_hex: digest("nar"),
                signature_count: 1,
            },
            &auth,
        );
        assert_eq!(oversized_log.unwrap_err().code, GatewayRejectCode::RequestTooLarge);
        assert_eq!(oversized_events.unwrap_err().code, GatewayRejectCode::RequestTooLarge);
        assert_eq!(oversized_upload.unwrap_err().code, GatewayRejectCode::RequestTooLarge);
    }

    #[test]
    fn store_transport_metadata_is_not_authoritative() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let unsigned = admit(
            GatewayOperation::UploadStoreObject {
                logical_path: "/nix/store/11111111111111111111111111111111-a".into(),
                nar_size_bytes: 1,
                nar_sha256_hex: digest("nar"),
                signature_count: 0,
            },
            &auth,
        );
        let malformed = admit(
            GatewayOperation::UploadStoreObject {
                logical_path: "/nix/store/11111111111111111111111111111111-a".into(),
                nar_size_bytes: 1,
                nar_sha256_hex: "claimed".into(),
                signature_count: 1,
            },
            &auth,
        );
        assert_eq!(unsigned.unwrap_err().code, GatewayRejectCode::StoreImportUntrusted);
        assert_eq!(malformed.unwrap_err().code, GatewayRejectCode::StoreImportUntrusted);
    }

    #[test]
    fn evaluator_text_and_arbitrary_command_are_not_concrete_derivations() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let mut operation = build_operation();
        let GatewayOperation::SubmitBuild {
            concrete_derivation_path,
            ..
        } = &mut operation
        else {
            unreachable!()
        };
        *concrete_derivation_path = "import <nixpkgs>".into();
        assert_eq!(admit(operation, &auth).unwrap_err().code, GatewayRejectCode::RequestInvalid);
    }

    #[test]
    fn duplicate_submission_recovers_only_exact_subject_scope_key_and_request() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let command = admit(build_operation(), &auth).unwrap();
        let GatewayCommand::SubmitRemoteAttempt {
            attempt_id,
            idempotency_key,
            request_identity_blake3,
            ..
        } = &command
        else {
            unreachable!()
        };
        let existing = ExistingGatewaySubmission {
            attempt_id: attempt_id.clone(),
            subject: auth.subject.clone(),
            account_scope: auth.account_scope.clone(),
            idempotency_key: idempotency_key.clone(),
            request_identity_blake3: request_identity_blake3.clone(),
        };
        let first = decide_gateway_submission(&command, &auth, None).unwrap();
        let duplicate = decide_gateway_submission(&command, &auth, Some(&existing)).unwrap();
        assert!(matches!(first, GatewaySubmissionDecision::Create { .. }));
        assert!(matches!(duplicate, GatewaySubmissionDecision::Recover { .. }));
    }

    #[test]
    fn idempotency_key_reuse_with_different_request_fails() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let command = admit(build_operation(), &auth).unwrap();
        let GatewayCommand::SubmitRemoteAttempt {
            attempt_id,
            idempotency_key,
            ..
        } = &command
        else {
            unreachable!()
        };
        let conflict = ExistingGatewaySubmission {
            attempt_id: attempt_id.clone(),
            subject: auth.subject.clone(),
            account_scope: auth.account_scope.clone(),
            idempotency_key: idempotency_key.clone(),
            request_identity_blake3: digest("other-request"),
        };
        let error = decide_gateway_submission(&command, &auth, Some(&conflict)).unwrap_err();
        assert_eq!(error.code, GatewayRejectCode::IdempotencyConflict);
    }

    #[test]
    fn reconnect_requires_existing_owned_attempt_and_status_authority() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let facts = GatewayReconnectFacts {
            attempt_id: attempt(),
            owner_subject: auth.subject.clone(),
            account_scope: auth.account_scope.clone(),
            exists: true,
        };
        assert!(decide_gateway_reconnect(&auth, &facts).is_ok());
        let mut foreign = facts.clone();
        foreign.owner_subject = "subject-b".into();
        assert_eq!(
            decide_gateway_reconnect(&auth, &foreign).unwrap_err().code,
            GatewayRejectCode::AttemptOwnerMismatch
        );
        let mut missing = facts;
        missing.exists = false;
        assert_eq!(decide_gateway_reconnect(&auth, &missing).unwrap_err().code, GatewayRejectCode::AttemptUnknown);
    }

    #[test]
    fn cursor_is_versioned_authenticated_scoped_and_fresh() {
        let key = [7_u8; 32];
        let attempt = attempt();
        let scope = || GatewayCursorScope {
            subject: "subject-a",
            account_scope: "project-a",
            attempt_id: &attempt,
            stream: "events",
        };
        let token = seal_gateway_cursor(GatewayCursorSealInput {
            scope: scope(),
            next_sequence: 2,
            expires_unix_s: NOW + 60,
            key: &key,
        })
        .unwrap();
        let verified = verify_gateway_cursor(&token, scope(), NOW, &key).unwrap();
        assert_eq!(verified.next_sequence, 2);
        assert!(!token.contains("subject-a"));
    }

    #[test]
    fn forged_stale_and_wrong_scope_cursors_fail_closed() {
        let key = [7_u8; 32];
        let attempt = attempt();
        let scope = |stream| GatewayCursorScope {
            subject: "subject-a",
            account_scope: "project-a",
            attempt_id: &attempt,
            stream,
        };
        let token = seal_gateway_cursor(GatewayCursorSealInput {
            scope: scope("events"),
            next_sequence: 2,
            expires_unix_s: NOW + 60,
            key: &key,
        })
        .unwrap();
        let forged = format!("{}0", &token[..token.len() - 1]);
        assert_eq!(
            verify_gateway_cursor(&forged, scope("events"), NOW, &key).unwrap_err().code,
            GatewayRejectCode::CursorInvalid
        );
        assert_eq!(
            verify_gateway_cursor(&token, scope("logs"), NOW, &key).unwrap_err().code,
            GatewayRejectCode::CursorScopeMismatch
        );
        assert_eq!(
            verify_gateway_cursor(&token, scope("events"), NOW + 60, &key).unwrap_err().code,
            GatewayRejectCode::CursorExpired
        );
    }

    #[test]
    fn completion_event_identity_is_stable_and_sequence_sensitive() {
        let first = plan_gateway_completion_event(GatewayCompletionPlanInput {
            attempt_id: attempt(),
            request_identity_blake3: digest("request"),
            terminal_class: GatewayTerminalClass::Completed,
            result_evidence_identity_blake3: Some(digest("result")),
            policy_identity_blake3: policy().policy_identity_blake3,
            sequence: 3,
        })
        .unwrap();
        let duplicate = first.clone();
        let next = plan_gateway_completion_event(GatewayCompletionPlanInput {
            attempt_id: attempt(),
            request_identity_blake3: digest("request"),
            terminal_class: GatewayTerminalClass::Completed,
            result_evidence_identity_blake3: Some(digest("result")),
            policy_identity_blake3: policy().policy_identity_blake3,
            sequence: 4,
        })
        .unwrap();
        assert_eq!(first.event_identity_blake3, duplicate.event_identity_blake3);
        assert_ne!(first.event_identity_blake3, next.event_identity_blake3);
    }

    #[test]
    fn completion_event_requires_external_signature_binding() {
        let event = plan_gateway_completion_event(GatewayCompletionPlanInput {
            attempt_id: attempt(),
            request_identity_blake3: digest("request"),
            terminal_class: GatewayTerminalClass::Failed,
            result_evidence_identity_blake3: None,
            policy_identity_blake3: policy().policy_identity_blake3,
            sequence: 1,
        })
        .unwrap();
        assert!(bind_gateway_completion_signature(event.clone(), "gateway-key".into(), "signature".into()).is_ok());
        assert_eq!(
            bind_gateway_completion_signature(event, "gateway-key".into(), "".into()).unwrap_err().code,
            GatewayRejectCode::CompletionEventInvalid
        );
    }

    #[test]
    fn evidence_is_redacted_bounded_and_keeps_non_claims() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let operation = build_operation();
        let evidence = gateway_evidence(GatewayEvidenceInput {
            policy: &policy(),
            authority: &auth,
            operation: &operation,
            attempt_id: Some(&attempt()),
            transferred_bytes: 42,
            stable_outcome: "admitted".into(),
        })
        .unwrap();
        let encoded = serde_json::to_string(&evidence).unwrap();
        assert_eq!(evidence.non_claims.len(), GATEWAY_NON_CLAIMS.len());
        assert_eq!(evidence.valence_observation.profile_id, VALENCE_BUILD_SERVICE_PROFILE_ID);
        assert_eq!(evidence.valence_observation.role, VALENCE_BUILD_SERVICE_RECORDED_ONLY_ROLE);
        assert_eq!(evidence.valence_observation.source_evidence_identity_blake3, evidence.evidence_identity_blake3);
        assert_eq!(evidence.valence_observation.non_claims, vec![VALENCE_BUILD_SERVICE_REQUIRED_NON_CLAIM.to_string()]);
        assert!(SECRET_MARKERS.iter().all(|marker| !encoded.to_ascii_lowercase().contains(marker)));
        assert!(is_blake3_hex(&evidence.evidence_identity_blake3));
    }

    #[test]
    fn sensitive_evidence_outcome_is_rejected() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let error = gateway_evidence(GatewayEvidenceInput {
            policy: &policy(),
            authority: &auth,
            operation: &build_operation(),
            attempt_id: None,
            transferred_bytes: 0,
            stable_outcome: "Bearer secret-material".into(),
        })
        .unwrap_err();
        assert_eq!(error.code, GatewayRejectCode::SensitiveEvidenceRejected);
    }

    #[test]
    fn operation_identity_distinguishes_digest_roles_without_claiming_truth() {
        let first = gateway_operation_identity(&build_operation()).unwrap();
        let mut second_operation = build_operation();
        let GatewayOperation::SubmitBuild {
            request_identity_blake3,
            ..
        } = &mut second_operation
        else {
            unreachable!()
        };
        *request_identity_blake3 = digest("request-b");
        let second = gateway_operation_identity(&second_operation).unwrap();
        assert_ne!(first, second);
        assert!(is_blake3_hex(&first));
    }

    #[test]
    fn cancellation_never_crosses_subject_ownership() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let error = admit(
            GatewayOperation::CancelAttempt {
                attempt_id: "attempt-a".into(),
                owner_subject: "subject-b".into(),
            },
            &auth,
        )
        .unwrap_err();
        assert_eq!(error.code, GatewayRejectCode::AttemptOwnerMismatch);
    }

    #[test]
    fn account_usage_cannot_cross_authority_scope() {
        let auth = authority(GatewayAuthoritySource::VerifiedUcan);
        let error = admit(
            GatewayOperation::UsageSummary {
                account_scope: "project-b".into(),
                item_count: 1,
            },
            &auth,
        )
        .unwrap_err();
        assert_eq!(error.code, GatewayRejectCode::AuthorityInvalid);
    }
}
