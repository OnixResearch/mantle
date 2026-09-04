//! Provider-neutral distributed build seams.
//!
//! This module is intentionally pure for the first distributed-build slice:
//! it defines realization-key inputs and deterministic key derivation without
//! contacting stores, schedulers, providers, or network services.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

mod external_batch;
mod remote_attempt;
mod remote_attempt_log;
mod remote_attempt_trellis;
mod remote_failure_debug;
mod remote_gateway;
mod remote_resources;
mod remote_telemetry;
mod remote_transfer;
pub mod snix_adapter;
pub use external_batch::*;
pub use remote_attempt::*;
pub use remote_attempt_log::*;
pub use remote_attempt_trellis::*;
pub use remote_failure_debug::*;
pub use remote_gateway::*;
pub use remote_resources::*;
pub use remote_telemetry::*;
pub use remote_transfer::*;
pub use snix_adapter::DerivationRealizer;
pub use snix_adapter::LocalBuildServiceRealizer;
#[cfg(test)]
use snix_adapter::REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH;
#[cfg(test)]
use snix_adapter::REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION;
pub use snix_adapter::RemoteBuildServiceAdapter;
pub use snix_adapter::RemoteBuildServiceDispatchError;
pub use snix_adapter::RemoteFailureDecision;
pub use snix_adapter::RemoteFirstBuildService;
pub use snix_adapter::classify_remote_build_service_failure;
pub use snix_adapter::validate_remote_build_service_request;

use crate::scheduling::ContentLocalityClass;
use crate::scheduling::EligiblePreferenceFacts;
use crate::scheduling::HardEligibilityFacts;
use crate::scheduling::IneligibleReason;
use crate::scheduling::ResourceFitClass;
use crate::scheduling::TransferCostClass;
use crate::scheduling::normalize_eligible_preference;

const MAX_READY_REMOTE_GOALS: usize = 4096;
const OPERATOR_SECRET_MARKERS: [&str; 5] = ["bearer ", "token=", "authorization", "secret", "password"];

pub use crunch_remote_core::DerivationKeyFacts;
pub use crunch_remote_core::InputClosureFact;
pub use crunch_remote_core::PlatformFacts;
pub use crunch_remote_core::RealizationKey;
pub use crunch_remote_core::RealizationKeyError;
pub use crunch_remote_core::RealizationKeyRequest;
pub use crunch_remote_core::RealizerProfileFacts;
pub use crunch_remote_core::RemoteBuildFallbackPolicy;
pub use crunch_remote_core::SandboxFacts;
pub use crunch_remote_core::StorePrefixFacts;
pub use crunch_remote_core::ToolchainFact;
pub use crunch_remote_core::VerifiedRemoteArtifact as VerifiedRealizationArtifact;

/// Derives deterministic realization keys from normalized Mantle facts.
pub trait RealizationKeyDeriver {
    fn derive_key(&self, request: &RealizationKeyRequest) -> Result<RealizationKey, RealizationKeyError>;
}

/// BLAKE3 realization-key adapter backed by the strict remote core.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRealizationKeyDeriver;

impl RealizationKeyDeriver for DefaultRealizationKeyDeriver {
    fn derive_key(&self, request: &RealizationKeyRequest) -> Result<RealizationKey, RealizationKeyError> {
        crunch_remote_core::derive_realization_key(request.clone())
    }
}

fn validate_sorted_unique_by<'a, T, F>(items: &'a [T], key: F) -> Result<(), ()>
where F: Fn(&'a T) -> &'a str {
    let mut previous: Option<&str> = None;
    for item in items {
        let current = key(item);
        if current.is_empty() || previous.is_some_and(|prior| prior >= current) {
            return Err(());
        }
        previous = Some(current);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactResolveRequest {
    pub key: RealizationKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveOutcome {
    Hit(VerifiedRealizationArtifact),
    Miss,
    Unavailable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishOutcome {
    pub published_outputs: u32,
    pub skipped_outputs: u32,
}

#[async_trait::async_trait]
pub trait ArtifactResolver: Send {
    async fn resolve(&mut self, request: &ArtifactResolveRequest) -> Result<ResolveOutcome, ArtifactAdapterError>;
}

#[async_trait::async_trait]
pub trait ArtifactPublisher: Send {
    async fn publish(
        &mut self,
        key: &RealizationKey,
        artifact: &VerifiedRealizationArtifact,
    ) -> Result<PublishOutcome, ArtifactAdapterError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedDiagnostic {
    CacheHit {
        key: RealizationKey,
        resolver: String,
    },
    CacheMiss {
        key: RealizationKey,
        resolver: String,
    },
    PublishSkipped {
        key: RealizationKey,
        publisher: String,
        reason: String,
    },
    LocalRealization {
        key: RealizationKey,
        realizer: String,
    },
    RemoteCandidate {
        key: RealizationKey,
        worker_id: String,
        verified: bool,
    },
    RemoteFallback {
        key: RealizationKey,
        reason: String,
    },
    VerificationRejected {
        key: RealizationKey,
        reason: String,
    },
}

impl DistributedDiagnostic {
    pub fn receipt(&self) -> String {
        match self {
            Self::CacheHit { key, resolver } => {
                format!("cache-hit key={} resolver={}", key.short(), redact_for_operator(resolver))
            }
            Self::CacheMiss { key, resolver } => {
                format!("cache-miss key={} resolver={}", key.short(), redact_for_operator(resolver))
            }
            Self::PublishSkipped { key, publisher, reason } => format!(
                "publish-skipped key={} publisher={} reason={}",
                key.short(),
                redact_for_operator(publisher),
                redact_for_operator(reason)
            ),
            Self::LocalRealization { key, realizer } => {
                format!("local-realization key={} realizer={}", key.short(), redact_for_operator(realizer))
            }
            Self::RemoteCandidate {
                key,
                worker_id,
                verified,
            } => format!(
                "remote-candidate key={} worker={} verified={}",
                key.short(),
                redact_for_operator(worker_id),
                verified
            ),
            Self::RemoteFallback { key, reason } => {
                format!("remote-fallback key={} reason={}", key.short(), redact_for_operator(reason))
            }
            Self::VerificationRejected { key, reason } => {
                format!("verification-rejected key={} reason={}", key.short(), redact_for_operator(reason))
            }
        }
    }
}

pub fn redact_for_operator(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let is_secret_bearing = OPERATOR_SECRET_MARKERS.iter().any(|marker| lower.contains(marker));
    if is_secret_bearing {
        "[REDACTED]".to_string()
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributedProfileConfig {
    pub resolvers: Vec<ResolverProfileConfig>,
    pub publishers: Vec<PublisherProfileConfig>,
    pub realizers: Vec<RealizerProfileConfig>,
}

impl Default for DistributedProfileConfig {
    fn default() -> Self {
        Self {
            resolvers: vec![],
            publishers: vec![],
            realizers: vec![RealizerProfileConfig {
                name: "local".to_string(),
                capabilities: vec!["local-build".to_string()],
                required: true,
                parameters: BTreeMap::new(),
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolverProfileConfig {
    pub name: String,
    pub capabilities: Vec<String>,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublisherProfileConfig {
    pub name: String,
    pub capabilities: Vec<String>,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizerProfileConfig {
    pub name: String,
    pub capabilities: Vec<String>,
    pub required: bool,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealizationPlan {
    RequireLocal,
    AllowRemote,
}

pub trait RealizationPolicy: Send + Sync {
    fn plan_for(&self, request: &RealizationKeyRequest) -> RealizationPlan;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalOnlyRealizationPolicy;

impl RealizationPolicy for LocalOnlyRealizationPolicy {
    fn plan_for(&self, _request: &RealizationKeyRequest) -> RealizationPlan {
        RealizationPlan::RequireLocal
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RemoteAllowedRealizationPolicy;

impl RealizationPolicy for RemoteAllowedRealizationPolicy {
    fn plan_for(&self, _request: &RealizationKeyRequest) -> RealizationPlan {
        RealizationPlan::AllowRemote
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteGoalAttachment {
    StartRemote {
        goal_key: String,
        realization_key: RealizationKey,
    },
    AttachToExisting {
        goal_key: String,
        realization_key: RealizationKey,
        owner_goal_key: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RemoteScheduleError {
    #[error("too many ready remote goals; maximum is {max}")]
    TooManyReadyGoals { max: usize },
    #[error(transparent)]
    Key(#[from] RealizationKeyError),
}

pub fn plan_remote_goal_attachments<D>(
    deriver: &D,
    ready: &[ReadyDerivationGoal],
    active_remote_jobs: &BTreeMap<RealizationKey, String>,
) -> Result<Vec<RemoteGoalAttachment>, RemoteScheduleError>
where
    D: RealizationKeyDeriver,
{
    if ready.len() > MAX_READY_REMOTE_GOALS {
        return Err(RemoteScheduleError::TooManyReadyGoals {
            max: MAX_READY_REMOTE_GOALS,
        });
    }
    let mut owners = active_remote_jobs.clone();
    let mut attachments = Vec::with_capacity(ready.len());
    for goal in ready {
        let realization_key = deriver.derive_key(&goal.request)?;
        if let Some(owner_goal_key) = owners.get(&realization_key) {
            attachments.push(RemoteGoalAttachment::AttachToExisting {
                goal_key: goal.goal_key.clone(),
                realization_key,
                owner_goal_key: owner_goal_key.clone(),
            });
        } else {
            owners.insert(realization_key.clone(), goal.goal_key.clone());
            attachments.push(RemoteGoalAttachment::StartRemote {
                goal_key: goal.goal_key.clone(),
                realization_key,
            });
        }
    }
    debug_assert_eq!(attachments.len(), ready.len());
    debug_assert!(attachments.len() <= MAX_READY_REMOTE_GOALS);
    Ok(attachments)
}

pub const HASH_NEGOTIATED_REMOTE_REALIZATION_VERSION: &str = "mantle-hash-negotiated-remote-realization-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteContentKind {
    Recipe,
    Blob,
    Directory,
    ProofInput,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RemoteContentIdentity {
    pub kind: RemoteContentKind,
    pub digest: String,
}

impl RemoteContentIdentity {
    pub fn new(kind: RemoteContentKind, bytes: &[u8]) -> Self {
        let digest = blake3::hash(bytes);
        Self {
            kind,
            digest: data_encoding::HEXLOWER.encode(digest.as_bytes()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteRealizationHandshakeRequest {
    pub version: String,
    pub realization_key: RealizationKey,
    pub recipe: RemoteContentIdentity,
    pub root_inputs: Vec<RemoteContentIdentity>,
    pub platform: PlatformFacts,
    pub worker_profile: String,
    pub declared_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRealizationHandshakeInput {
    pub realization_key: RealizationKey,
    pub recipe: RemoteContentIdentity,
    pub root_inputs: Vec<RemoteContentIdentity>,
    pub platform: PlatformFacts,
    pub worker_profile: String,
    pub declared_capabilities: Vec<String>,
}

impl RemoteRealizationHandshakeRequest {
    pub fn new(input: RemoteRealizationHandshakeInput) -> Self {
        Self {
            version: HASH_NEGOTIATED_REMOTE_REALIZATION_VERSION.to_string(),
            realization_key: input.realization_key,
            recipe: input.recipe,
            root_inputs: input.root_inputs,
            platform: input.platform,
            worker_profile: input.worker_profile,
            declared_capabilities: input.declared_capabilities,
        }
    }

    pub fn negotiated_inputs(&self) -> Vec<RemoteContentIdentity> {
        let mut inputs = vec![self.recipe.clone()];
        inputs.extend(self.root_inputs.clone());
        inputs.sort();
        inputs.dedup();
        inputs
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteRealizationNegotiationResponse {
    MissingContent { missing: Vec<RemoteContentIdentity> },
    UnsupportedProfile { worker_profile: String },
    CapabilityDenied { capability: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteContentTransfer {
    pub identity: RemoteContentIdentity,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOutputDigest {
    pub output_name: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteRealizationReceipt {
    pub version: String,
    pub realization_key: RealizationKey,
    pub worker_profile: String,
    pub negotiated_inputs: Vec<RemoteContentIdentity>,
    pub declared_capabilities: Vec<String>,
    pub observed_capabilities: Vec<String>,
    pub outputs: Vec<RemoteOutputDigest>,
}

impl RemoteRealizationReceipt {
    pub fn validate(&self) -> Result<(), RemoteRealizationProtocolError> {
        if self.version != HASH_NEGOTIATED_REMOTE_REALIZATION_VERSION {
            return Err(RemoteRealizationProtocolError::UnsupportedVersion {
                version: self.version.clone(),
            });
        }
        if self.negotiated_inputs.is_empty() || self.outputs.is_empty() {
            return Err(RemoteRealizationProtocolError::MissingContent);
        }
        let mut identities = std::collections::BTreeSet::new();
        for identity in &self.negotiated_inputs {
            if identity.digest.is_empty() || !identities.insert(identity) {
                return Err(RemoteRealizationProtocolError::MissingContent);
            }
        }
        validate_sorted_unique_by(&self.outputs, |output| output.output_name.as_str())
            .map_err(|_| RemoteRealizationProtocolError::MissingContent)?;
        if self.outputs.iter().any(|output| output.digest.is_empty()) {
            return Err(RemoteRealizationProtocolError::MissingContent);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RemoteRealizationProtocolError {
    #[error("unsupported remote realization handshake version {version}")]
    UnsupportedVersion { version: String },
    #[error("unsupported remote worker profile {worker_profile}")]
    UnsupportedProfile { worker_profile: String },
    #[error("remote realization capability denied: {capability}")]
    CapabilityDenied { capability: String },
    #[error("remote realization missing requested content")]
    MissingContent,
    #[error("remote realization transfer digest mismatch for {digest}")]
    DigestMismatch { digest: String },
}

#[derive(Debug, Clone)]
pub struct InMemoryRemoteRealizationWorker {
    worker_profile: String,
    capabilities: Vec<String>,
    content: BTreeMap<RemoteContentIdentity, Vec<u8>>,
}

impl InMemoryRemoteRealizationWorker {
    pub fn new(worker_profile: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            worker_profile: worker_profile.into(),
            capabilities,
            content: BTreeMap::new(),
        }
    }

    pub fn insert_content(&mut self, kind: RemoteContentKind, bytes: Vec<u8>) -> RemoteContentIdentity {
        let identity = RemoteContentIdentity::new(kind, &bytes);
        self.content.insert(identity.clone(), bytes);
        identity
    }

    pub fn negotiate(
        &self,
        request: &RemoteRealizationHandshakeRequest,
    ) -> Result<RemoteRealizationNegotiationResponse, RemoteRealizationProtocolError> {
        self.validate_request_profile_and_capabilities(request)?;
        let missing = request
            .negotiated_inputs()
            .into_iter()
            .filter(|identity| !self.content.contains_key(identity))
            .collect();
        Ok(RemoteRealizationNegotiationResponse::MissingContent { missing })
    }

    pub fn accept_transfer(&mut self, transfer: RemoteContentTransfer) -> Result<(), RemoteRealizationProtocolError> {
        let actual = RemoteContentIdentity::new(transfer.identity.kind, &transfer.bytes);
        if actual.digest != transfer.identity.digest {
            return Err(RemoteRealizationProtocolError::DigestMismatch {
                digest: transfer.identity.digest,
            });
        }
        self.content.insert(transfer.identity, transfer.bytes);
        Ok(())
    }

    pub fn realize(
        &self,
        request: &RemoteRealizationHandshakeRequest,
    ) -> Result<RemoteRealizationReceipt, RemoteRealizationProtocolError> {
        self.validate_request_profile_and_capabilities(request)?;
        let negotiated_inputs = request.negotiated_inputs();
        if negotiated_inputs.iter().any(|identity| !self.content.contains_key(identity)) {
            return Err(RemoteRealizationProtocolError::MissingContent);
        }
        let mut hasher = blake3::Hasher::new();
        for identity in &negotiated_inputs {
            hasher.update(identity.digest.as_bytes());
        }
        let output_digest = data_encoding::HEXLOWER.encode(hasher.finalize().as_bytes());
        let mut declared_capabilities = request.declared_capabilities.clone();
        declared_capabilities.sort();
        declared_capabilities.dedup();
        let mut negotiated_inputs = negotiated_inputs;
        negotiated_inputs.sort();
        negotiated_inputs.dedup();
        let receipt = RemoteRealizationReceipt {
            version: HASH_NEGOTIATED_REMOTE_REALIZATION_VERSION.to_string(),
            realization_key: request.realization_key.clone(),
            worker_profile: self.worker_profile.clone(),
            negotiated_inputs,
            declared_capabilities: declared_capabilities.clone(),
            observed_capabilities: declared_capabilities,
            outputs: vec![RemoteOutputDigest {
                output_name: "out".to_string(),
                digest: output_digest,
            }],
        };
        receipt.validate()?;
        Ok(receipt)
    }

    fn validate_request_profile_and_capabilities(
        &self,
        request: &RemoteRealizationHandshakeRequest,
    ) -> Result<(), RemoteRealizationProtocolError> {
        if request.version != HASH_NEGOTIATED_REMOTE_REALIZATION_VERSION {
            return Err(RemoteRealizationProtocolError::UnsupportedVersion {
                version: request.version.clone(),
            });
        }
        if request.worker_profile != self.worker_profile {
            return Err(RemoteRealizationProtocolError::UnsupportedProfile {
                worker_profile: request.worker_profile.clone(),
            });
        }
        let worker_capabilities = self.capabilities.iter().collect::<std::collections::BTreeSet<_>>();
        if let Some(capability) =
            request.declared_capabilities.iter().find(|capability| !worker_capabilities.contains(capability))
        {
            return Err(RemoteRealizationProtocolError::CapabilityDenied {
                capability: capability.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkerMetadata {
    pub worker_id: String,
    pub realizer_profile: String,
    pub attestation: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOutputReference {
    pub output_name: String,
    pub store_path: String,
    pub nar_sha256_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteVerificationStatus {
    Verified,
    Unverified { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteRealizationCandidate {
    pub key: RealizationKey,
    pub log: String,
    pub outputs: Vec<RemoteOutputReference>,
    pub worker: RemoteWorkerMetadata,
    pub verification: RemoteVerificationStatus,
}

impl RemoteRealizationCandidate {
    pub fn verify_for_persistence(
        &self,
        expected_key: &RealizationKey,
        expected_outputs: &[String],
    ) -> Result<(), RemoteVerificationError> {
        if &self.key != expected_key {
            return Err(RemoteVerificationError::KeyMismatch);
        }
        if self.log.is_empty() {
            return Err(RemoteVerificationError::MissingLog);
        }
        if self.worker.worker_id.is_empty() || self.worker.realizer_profile.is_empty() {
            return Err(RemoteVerificationError::MissingWorkerMetadata);
        }
        match &self.verification {
            RemoteVerificationStatus::Verified => {}
            RemoteVerificationStatus::Unverified { reason } => {
                return Err(RemoteVerificationError::Unverified { reason: reason.clone() });
            }
        }
        let expected: std::collections::BTreeSet<_> = expected_outputs.iter().cloned().collect();
        let actual: std::collections::BTreeSet<_> =
            self.outputs.iter().map(|output| output.output_name.clone()).collect();
        if actual != expected {
            return Err(RemoteVerificationError::OutputMismatch);
        }
        if self.outputs.iter().any(|output| output.store_path.is_empty() || output.nar_sha256_hex.is_empty()) {
            return Err(RemoteVerificationError::OutputMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RemoteVerificationError {
    #[error("remote candidate key does not match requested realization")]
    KeyMismatch,
    #[error("remote candidate is unverified: {reason}")]
    Unverified { reason: String },
    #[error("remote candidate omitted build log evidence")]
    MissingLog,
    #[error("remote candidate omitted worker metadata")]
    MissingWorkerMetadata,
    #[error("remote candidate outputs do not match expected outputs")]
    OutputMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealizationDispatchDecision {
    UseResolvedArtifact(VerifiedRealizationArtifact),
    BuildLocally,
}

pub async fn resolve_before_dispatch(
    resolvers: &mut [&mut dyn ArtifactResolver],
    request: &ArtifactResolveRequest,
) -> Result<RealizationDispatchDecision, ArtifactAdapterError> {
    for resolver in resolvers.iter_mut() {
        match resolver.resolve(request).await? {
            ResolveOutcome::Hit(artifact) => return Ok(RealizationDispatchDecision::UseResolvedArtifact(artifact)),
            ResolveOutcome::Miss => continue,
            ResolveOutcome::Unavailable { .. } => continue,
        }
    }
    Ok(RealizationDispatchDecision::BuildLocally)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyDerivationGoal {
    pub goal_key: String,
    pub request: RealizationKeyRequest,
}

/// Results of the existing hard route checks, normalized without provider
/// names or transport-specific state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteRouteEligibilityChecks {
    pub capability_allowed: bool,
    pub output_trust_allowed: bool,
    pub upload_allowed: bool,
    pub network_allowed: bool,
    pub store_prefix_matches: bool,
    pub hard_resource_fit: bool,
}

/// Provider-neutral preference facts admitted only after all hard route checks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RemoteRoutePreferenceFacts {
    pub resource_fit: ResourceFitClass,
    pub content_locality: ContentLocalityClass,
    pub transfer_cost: TransferCostClass,
}

/// Keep route eligibility as a hard gate before scheduler preference.
///
/// r[impl build_scheduling.resource_locality_preference]
pub fn normalize_remote_route_scheduling_facts(
    checks: RemoteRouteEligibilityChecks,
    preference: RemoteRoutePreferenceFacts,
) -> Result<EligiblePreferenceFacts, IneligibleReason> {
    let eligibility = HardEligibilityFacts {
        capability_allowed: checks.capability_allowed,
        output_trust_allowed: checks.output_trust_allowed,
        upload_allowed: checks.upload_allowed,
        network_allowed: checks.network_allowed,
        store_prefix_matches: checks.store_prefix_matches,
        hard_resource_fit: checks.hard_resource_fit,
    };
    let normalized = normalize_eligible_preference(
        eligibility,
        preference.resource_fit,
        preference.content_locality,
        preference.transfer_cost,
    )?;
    debug_assert_eq!(normalized.resource_fit, preference.resource_fit);
    debug_assert_eq!(normalized.content_locality, preference.content_locality);
    Ok(normalized)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRealization {
    pub goal_key: String,
    pub plan: RealizationPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealizationJobCapacity {
    pub running_jobs: u32,
    pub max_jobs: u32,
}

/// Apply max-job and dedup ownership to a priority-ranked ready snapshot.
///
/// The caller owns deterministic priority ordering; this route seam preserves
/// that order and cannot make an ineligible route eligible.
pub fn select_ready_realizations<P>(
    policy: &P,
    ready: &[ReadyDerivationGoal],
    capacity: RealizationJobCapacity,
) -> Vec<ScheduledRealization>
where
    P: RealizationPolicy,
{
    let available_slots = capacity.max_jobs.saturating_sub(capacity.running_jobs);
    let available_slots = match usize::try_from(available_slots) {
        Ok(available_slots) => available_slots.min(ready.len()),
        Err(_) => ready.len(),
    };
    let mut seen = std::collections::BTreeSet::new();
    ready
        .iter()
        .filter(|goal| seen.insert(goal.goal_key.clone()))
        .take(available_slots)
        .map(|goal| ScheduledRealization {
            goal_key: goal.goal_key.clone(),
            plan: policy.plan_for(&goal.request),
        })
        .collect()
}

/// Local test adapter that models a resolver/publisher over verified PathInfo
/// records without naming a remote provider or transport.
#[derive(Debug, Clone, Default)]
pub struct InMemoryArtifactAdapter {
    artifacts: BTreeMap<RealizationKey, VerifiedRealizationArtifact>,
}

impl InMemoryArtifactAdapter {
    pub fn insert_verified(
        &mut self,
        key: RealizationKey,
        artifact: VerifiedRealizationArtifact,
    ) -> Option<VerifiedRealizationArtifact> {
        self.artifacts.insert(key, artifact)
    }

    pub fn contains_key(&self, key: &RealizationKey) -> bool {
        self.artifacts.contains_key(key)
    }
}

#[async_trait::async_trait]
impl ArtifactResolver for InMemoryArtifactAdapter {
    async fn resolve(&mut self, request: &ArtifactResolveRequest) -> Result<ResolveOutcome, ArtifactAdapterError> {
        Ok(self.artifacts.get(&request.key).cloned().map(ResolveOutcome::Hit).unwrap_or(ResolveOutcome::Miss))
    }
}

#[async_trait::async_trait]
impl ArtifactPublisher for InMemoryArtifactAdapter {
    async fn publish(
        &mut self,
        key: &RealizationKey,
        artifact: &VerifiedRealizationArtifact,
    ) -> Result<PublishOutcome, ArtifactAdapterError> {
        let published_outputs: u32 =
            artifact.outputs.len().try_into().map_err(|_| ArtifactAdapterError::TooManyOutputs)?;
        let skipped_outputs = if self.artifacts.contains_key(key) {
            published_outputs
        } else {
            0
        };
        self.artifacts.insert(key.clone(), artifact.clone());
        Ok(PublishOutcome {
            published_outputs: published_outputs.saturating_sub(skipped_outputs),
            skipped_outputs,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArtifactAdapterError {
    #[error("verified artifact must contain at least one output")]
    EmptyArtifact,
    #[error("substitution report references unknown output {output}")]
    UnknownSubstitutionOutput { output: String },
    #[error("artifact contains too many outputs to report as u32")]
    TooManyOutputs,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_ne;
    use snix_build::buildservice::BuildService;

    use super::*;

    const TEST_PROFILE_VERSION: u32 = 1;
    const TEST_HARD_BLOCKER_CASE_COUNT: usize = 6;
    const TEST_OUTPUT_SIZE_BYTES: u64 = 128;
    const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn base_request() -> RealizationKeyRequest {
        RealizationKeyRequest {
            derivation: DerivationKeyFacts {
                identity: "/crunch/store/11111111111111111111111111111111-example.drv".to_string(),
                builder: "/crunch/store/22222222222222222222222222222222-builder/bin/build".to_string(),
                args: vec!["--build".to_string(), "example".to_string()],
                outputs: vec!["out".to_string()],
            },
            input_closure: vec![
                InputClosureFact {
                    store_path: "/crunch/store/33333333333333333333333333333333-libc".to_string(),
                    nar_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    references: vec![],
                },
                InputClosureFact {
                    store_path: "/crunch/store/44444444444444444444444444444444-tool".to_string(),
                    nar_hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                    references: vec!["/crunch/store/33333333333333333333333333333333-libc".to_string()],
                },
            ],
            platform: PlatformFacts {
                system: "x86_64-linux".to_string(),
                cpu: "x86_64".to_string(),
                os: "linux".to_string(),
            },
            toolchains: vec![ToolchainFact {
                name: "cc".to_string(),
                store_path: "/crunch/store/55555555555555555555555555555555-gcc".to_string(),
                digest: Some("sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string()),
            }],
            sandbox: SandboxFacts {
                hermeticity: "strict".to_string(),
                network_allowed: false,
                fixed_output: false,
            },
            environment: BTreeMap::from([
                ("PATH".to_string(), b"/crunch/store/55555555555555555555555555555555-gcc/bin".to_vec()),
                ("SOURCE_DATE_EPOCH".to_string(), b"1".to_vec()),
            ]),
            store: StorePrefixFacts {
                logical_prefix: "/crunch/store".to_string(),
                output_prefix: "/tmp/crunch-store".to_string(),
            },
            realizer_profile: RealizerProfileFacts {
                name: "local-sandbox".to_string(),
                version: 1,
                capabilities: vec!["local".to_string(), "sandbox".to_string()],
                parameters: BTreeMap::from([(
                    "sandbox-shell".to_string(),
                    "/crunch/store/static-busybox/bin/sh".to_string(),
                )]),
            },
        }
    }

    fn key(request: &RealizationKeyRequest) -> RealizationKey {
        DefaultRealizationKeyDeriver.derive_key(request).unwrap()
    }

    fn concrete_build_request() -> snix_build::buildservice::BuildRequest {
        snix_build::buildservice::BuildRequest {
            command_args: vec!["/bin/build".to_string(), "arg".to_string()],
            outputs: vec![std::path::PathBuf::from("mantle/store/example-out")],
            ..Default::default()
        }
    }

    fn build_result(log: &str) -> snix_build::buildservice::BuildResult {
        snix_build::buildservice::BuildResult {
            outputs: vec![snix_build::buildservice::BuildOutput {
                node: snix_castore::Node::Symlink {
                    target: log.try_into().unwrap(),
                },
                output_needles: std::collections::BTreeSet::new(),
            }],
            log: Some(log.to_string()),
        }
    }

    fn verified_artifact() -> VerifiedRealizationArtifact {
        let output = crunch_remote_core::RemoteOutputFact {
            name: "out".to_string(),
            logical_path: "/crunch/store/artifact".to_string(),
            content_digest_blake3: TEST_DIGEST.to_string(),
            size_bytes: TEST_OUTPUT_SIZE_BYTES,
            artifact_attestation_blake3: TEST_DIGEST.to_string(),
            path_info_blake3: Some(TEST_DIGEST.to_string()),
            substitution: None,
        };
        VerifiedRealizationArtifact::new(BTreeMap::from([("out".to_string(), output)])).unwrap()
    }

    #[test]
    fn in_memory_adapter_reports_miss_then_hit_after_publish() {
        let mut adapter = InMemoryArtifactAdapter::default();
        let key = key(&base_request());
        let artifact = verified_artifact();

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            assert_eq!(
                adapter.resolve(&ArtifactResolveRequest { key: key.clone() }).await.unwrap(),
                ResolveOutcome::Miss
            );
            assert_eq!(adapter.publish(&key, &artifact).await.unwrap(), PublishOutcome {
                published_outputs: 1,
                skipped_outputs: 0,
            });
            assert!(matches!(
                adapter.resolve(&ArtifactResolveRequest { key }).await.unwrap(),
                ResolveOutcome::Hit(hit) if hit == artifact
            ));
        });
    }

    #[test]
    fn in_memory_publisher_reports_idempotent_skip() {
        let mut adapter = InMemoryArtifactAdapter::default();
        let key = key(&base_request());
        let artifact = verified_artifact();

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            adapter.publish(&key, &artifact).await.unwrap();
            assert_eq!(adapter.publish(&key, &artifact).await.unwrap(), PublishOutcome {
                published_outputs: 0,
                skipped_outputs: 1,
            });
        });
    }

    #[test]
    fn verified_artifact_rejects_empty_outputs() {
        let error = VerifiedRealizationArtifact::new(BTreeMap::new()).unwrap_err();
        assert_eq!(error.code(), "verified-artifact-empty");
        assert!(matches!(error, crunch_remote_core::RemoteCoreError::Invalid { .. }));
    }

    #[test]
    fn verified_artifact_rejects_output_name_mismatch() {
        let output = verified_artifact().outputs.remove("out").unwrap();
        let error = VerifiedRealizationArtifact::new(BTreeMap::from([("missing".to_string(), output)])).unwrap_err();
        assert_eq!(error.code(), "verified-artifact-output-name-mismatch");
        assert!(matches!(error, crunch_remote_core::RemoteCoreError::Invalid { .. }));
    }

    fn remote_request(
        recipe: RemoteContentIdentity,
        root_inputs: Vec<RemoteContentIdentity>,
    ) -> RemoteRealizationHandshakeRequest {
        RemoteRealizationHandshakeRequest::new(RemoteRealizationHandshakeInput {
            realization_key: key(&base_request()),
            recipe,
            root_inputs,
            platform: PlatformFacts {
                system: "x86_64-linux".to_string(),
                cpu: "x86_64".to_string(),
                os: "linux".to_string(),
            },
            worker_profile: "fake-worker-v1".to_string(),
            declared_capabilities: vec!["sandbox".to_string(), "write-output".to_string()],
        })
    }

    fn remote_worker() -> InMemoryRemoteRealizationWorker {
        InMemoryRemoteRealizationWorker::new("fake-worker-v1", vec!["sandbox".to_string(), "write-output".to_string()])
    }

    #[test]
    fn hash_negotiated_remote_realization_all_present_executes_without_transfer() {
        let mut worker = remote_worker();
        let recipe = worker.insert_content(RemoteContentKind::Recipe, b"drv recipe".to_vec());
        let input = worker.insert_content(RemoteContentKind::Directory, b"source tree".to_vec());
        let request = remote_request(recipe.clone(), vec![input.clone()]);

        assert_eq!(worker.negotiate(&request).unwrap(), RemoteRealizationNegotiationResponse::MissingContent {
            missing: vec![]
        });
        let receipt = worker.realize(&request).unwrap();
        assert_eq!(receipt.version, HASH_NEGOTIATED_REMOTE_REALIZATION_VERSION);
        assert_eq!(receipt.realization_key, request.realization_key);
        assert_eq!(receipt.negotiated_inputs, vec![recipe, input]);
        assert_eq!(receipt.declared_capabilities, vec!["sandbox".to_string(), "write-output".to_string()]);
        assert_eq!(receipt.outputs.len(), 1);
    }

    #[test]
    fn hash_negotiated_remote_realization_requests_missing_hashes_then_verifies_transfer() {
        let mut worker = remote_worker();
        let recipe = worker.insert_content(RemoteContentKind::Recipe, b"drv recipe".to_vec());
        let input_bytes = b"source tree".to_vec();
        let input = RemoteContentIdentity::new(RemoteContentKind::Directory, &input_bytes);
        let request = remote_request(recipe.clone(), vec![input.clone()]);

        assert_eq!(worker.negotiate(&request).unwrap(), RemoteRealizationNegotiationResponse::MissingContent {
            missing: vec![input.clone()]
        });
        assert_eq!(worker.realize(&request).unwrap_err(), RemoteRealizationProtocolError::MissingContent);
        worker
            .accept_transfer(RemoteContentTransfer {
                identity: input.clone(),
                bytes: input_bytes,
            })
            .unwrap();
        let receipt = worker.realize(&request).unwrap();
        assert_eq!(receipt.negotiated_inputs, vec![recipe, input]);
    }

    #[test]
    fn hash_negotiated_remote_realization_rejects_digest_mismatch_before_execution() {
        let mut worker = remote_worker();
        let recipe = worker.insert_content(RemoteContentKind::Recipe, b"drv recipe".to_vec());
        let input = RemoteContentIdentity::new(RemoteContentKind::Blob, b"expected bytes");
        let request = remote_request(recipe, vec![input.clone()]);

        assert_eq!(
            worker
                .accept_transfer(RemoteContentTransfer {
                    identity: input.clone(),
                    bytes: b"tampered bytes".to_vec(),
                })
                .unwrap_err(),
            RemoteRealizationProtocolError::DigestMismatch { digest: input.digest }
        );
        assert_eq!(worker.realize(&request).unwrap_err(), RemoteRealizationProtocolError::MissingContent);
    }

    #[test]
    fn hash_negotiated_remote_realization_rejects_unsupported_capability_and_profile() {
        let mut worker = remote_worker();
        let recipe = worker.insert_content(RemoteContentKind::Recipe, b"drv recipe".to_vec());
        let input = worker.insert_content(RemoteContentKind::Directory, b"source tree".to_vec());
        let mut request = remote_request(recipe, vec![input]);
        request.declared_capabilities.push("network".to_string());

        assert_eq!(worker.negotiate(&request).unwrap_err(), RemoteRealizationProtocolError::CapabilityDenied {
            capability: "network".to_string()
        });
        request.declared_capabilities = vec!["sandbox".to_string()];
        request.worker_profile = "gpu-worker".to_string();
        assert_eq!(worker.negotiate(&request).unwrap_err(), RemoteRealizationProtocolError::UnsupportedProfile {
            worker_profile: "gpu-worker".to_string()
        });
    }

    fn remote_candidate(key: RealizationKey) -> RemoteRealizationCandidate {
        RemoteRealizationCandidate {
            key,
            log: "remote build log".to_string(),
            outputs: vec![RemoteOutputReference {
                output_name: "out".to_string(),
                store_path: "/crunch/store/abc-out".to_string(),
                nar_sha256_hex: "0123456789abcdef".to_string(),
            }],
            worker: RemoteWorkerMetadata {
                worker_id: "worker-1".to_string(),
                realizer_profile: "fake-remote@1".to_string(),
                attestation: BTreeMap::new(),
            },
            verification: RemoteVerificationStatus::Verified,
        }
    }

    #[test]
    fn diagnostic_receipts_cover_operator_events_and_redact_secrets() {
        let key = key(&base_request());
        let diagnostics = [
            DistributedDiagnostic::CacheHit {
                key: key.clone(),
                resolver: "local-cache".to_string(),
            },
            DistributedDiagnostic::CacheMiss {
                key: key.clone(),
                resolver: "remote-cache token=SHOULD_NOT_LEAK".to_string(),
            },
            DistributedDiagnostic::PublishSkipped {
                key: key.clone(),
                publisher: "publisher".to_string(),
                reason: "authorization Bearer SHOULD_NOT_LEAK".to_string(),
            },
            DistributedDiagnostic::LocalRealization {
                key: key.clone(),
                realizer: "local".to_string(),
            },
            DistributedDiagnostic::RemoteCandidate {
                key: key.clone(),
                worker_id: "worker-secret".to_string(),
                verified: true,
            },
            DistributedDiagnostic::RemoteFallback {
                key: key.clone(),
                reason: "password=SHOULD_NOT_LEAK".to_string(),
            },
            DistributedDiagnostic::VerificationRejected {
                key,
                reason: "digest mismatch".to_string(),
            },
        ];

        let receipts: Vec<_> = diagnostics.iter().map(DistributedDiagnostic::receipt).collect();
        for expected in [
            "cache-hit",
            "cache-miss",
            "publish-skipped",
            "local-realization",
            "remote-candidate",
            "remote-fallback",
            "verification-rejected",
        ] {
            assert!(receipts.iter().any(|receipt| receipt.contains(expected)), "missing {expected}");
        }
        let joined = receipts.join("\n");
        assert!(!joined.contains("SHOULD_NOT_LEAK"));
        assert!(joined.contains("[REDACTED]"));
    }

    #[test]
    fn distributed_profile_default_is_provider_neutral_local_only() {
        let config = DistributedProfileConfig::default();
        assert!(config.resolvers.is_empty());
        assert!(config.publishers.is_empty());
        assert_eq!(config.realizers.len(), 1);
        assert_eq!(config.realizers[0].name, "local");
        let serialized = serde_json::to_string(&config).unwrap();
        for forbidden in ["ssh", "http", "https", "s3", "gcs", "nomad", "buildfarm"] {
            assert!(!serialized.contains(forbidden), "default config named provider {forbidden}");
        }
    }

    #[test]
    fn distributed_profile_config_round_trips_capabilities_and_parameters() {
        let config = DistributedProfileConfig {
            resolvers: vec![ResolverProfileConfig {
                name: "resolver-a".to_string(),
                capabilities: vec!["pathinfo".to_string()],
                parameters: BTreeMap::from([("priority".to_string(), "10".to_string())]),
            }],
            publishers: vec![PublisherProfileConfig {
                name: "publisher-a".to_string(),
                capabilities: vec!["nar-export".to_string()],
                parameters: BTreeMap::new(),
            }],
            realizers: vec![RealizerProfileConfig {
                name: "realizer-a".to_string(),
                capabilities: vec!["remote-build".to_string()],
                required: false,
                parameters: BTreeMap::from([("system".to_string(), "x86_64-linux".to_string())]),
            }],
        };

        let encoded = serde_json::to_string(&config).unwrap();
        let decoded: DistributedProfileConfig = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn remote_candidate_verification_accepts_verified_outputs() {
        let key = key(&base_request());
        remote_candidate(key.clone()).verify_for_persistence(&key, &["out".to_string()]).unwrap();
    }

    #[test]
    fn remote_candidate_verification_rejects_key_mismatch() {
        let actual_key = key(&base_request());
        let mut changed_request = base_request();
        changed_request.derivation.identity.push_str("-different");
        let expected_key = key(&changed_request);
        assert_eq!(
            remote_candidate(actual_key)
                .verify_for_persistence(&expected_key, &["out".to_string()])
                .unwrap_err(),
            RemoteVerificationError::KeyMismatch
        );
    }

    #[test]
    fn remote_candidate_verification_rejects_unverified_status() {
        let key = key(&base_request());
        let mut candidate = remote_candidate(key.clone());
        candidate.verification = RemoteVerificationStatus::Unverified {
            reason: "digest mismatch".to_string(),
        };
        assert_eq!(
            candidate.verify_for_persistence(&key, &["out".to_string()]).unwrap_err(),
            RemoteVerificationError::Unverified {
                reason: "digest mismatch".to_string(),
            }
        );
    }

    #[test]
    fn remote_candidate_verification_rejects_missing_log_or_metadata() {
        let key = key(&base_request());
        let mut missing_log = remote_candidate(key.clone());
        missing_log.log.clear();
        assert_eq!(
            missing_log.verify_for_persistence(&key, &["out".to_string()]).unwrap_err(),
            RemoteVerificationError::MissingLog
        );

        let mut missing_metadata = remote_candidate(key.clone());
        missing_metadata.worker.worker_id.clear();
        assert_eq!(
            missing_metadata.verify_for_persistence(&key, &["out".to_string()]).unwrap_err(),
            RemoteVerificationError::MissingWorkerMetadata
        );
    }

    #[test]
    fn remote_candidate_verification_rejects_output_mismatch() {
        let key = key(&base_request());
        assert_eq!(
            remote_candidate(key.clone()).verify_for_persistence(&key, &["dev".to_string()]).unwrap_err(),
            RemoteVerificationError::OutputMismatch
        );
    }

    // r[verify build_scheduling.resource_locality_preference]
    #[test]
    fn scheduler_route_facts_normalize_only_after_every_hard_check() {
        let eligible = RemoteRouteEligibilityChecks {
            capability_allowed: true,
            output_trust_allowed: true,
            upload_allowed: true,
            network_allowed: true,
            store_prefix_matches: true,
            hard_resource_fit: true,
        };
        let preferred = RemoteRoutePreferenceFacts {
            resource_fit: ResourceFitClass::Exact,
            content_locality: ContentLocalityClass::FullyPresent,
            transfer_cost: TransferCostClass::None,
        };

        let normalized = normalize_remote_route_scheduling_facts(eligible, preferred).unwrap();

        assert_eq!(normalized.resource_fit, ResourceFitClass::Exact);
        assert_eq!(normalized.content_locality, ContentLocalityClass::FullyPresent);
        assert_eq!(normalized.transfer_cost, TransferCostClass::None);
    }

    // r[verify build_scheduling.resource_locality_preference]
    #[test]
    fn scheduler_route_preference_cannot_bypass_any_hard_blocker() {
        let eligible = RemoteRouteEligibilityChecks {
            capability_allowed: true,
            output_trust_allowed: true,
            upload_allowed: true,
            network_allowed: true,
            store_prefix_matches: true,
            hard_resource_fit: true,
        };
        let preferred = RemoteRoutePreferenceFacts {
            resource_fit: ResourceFitClass::Exact,
            content_locality: ContentLocalityClass::FullyPresent,
            transfer_cost: TransferCostClass::None,
        };
        let cases = [
            (
                RemoteRouteEligibilityChecks {
                    capability_allowed: false,
                    ..eligible
                },
                IneligibleReason::MissingCapability,
            ),
            (
                RemoteRouteEligibilityChecks {
                    output_trust_allowed: false,
                    ..eligible
                },
                IneligibleReason::OutputTrustRejected,
            ),
            (
                RemoteRouteEligibilityChecks {
                    upload_allowed: false,
                    ..eligible
                },
                IneligibleReason::UploadPolicyRejected,
            ),
            (
                RemoteRouteEligibilityChecks {
                    network_allowed: false,
                    ..eligible
                },
                IneligibleReason::NetworkPolicyRejected,
            ),
            (
                RemoteRouteEligibilityChecks {
                    store_prefix_matches: false,
                    ..eligible
                },
                IneligibleReason::StorePrefixMismatch,
            ),
            (
                RemoteRouteEligibilityChecks {
                    hard_resource_fit: false,
                    ..eligible
                },
                IneligibleReason::HardResourceMismatch,
            ),
        ];

        for (checks, expected) in cases {
            assert_eq!(normalize_remote_route_scheduling_facts(checks, preferred), Err(expected));
        }
        assert_eq!(cases.len(), TEST_HARD_BLOCKER_CASE_COUNT);
    }

    #[test]
    fn scheduler_selection_preserves_max_jobs_budget() {
        let mut second = base_request();
        second.derivation.identity = "second.drv".to_string();
        let ready = vec![
            ReadyDerivationGoal {
                goal_key: "a".to_string(),
                request: base_request(),
            },
            ReadyDerivationGoal {
                goal_key: "b".to_string(),
                request: second,
            },
        ];

        let scheduled = select_ready_realizations(&LocalOnlyRealizationPolicy, &ready, RealizationJobCapacity {
            running_jobs: 1,
            max_jobs: 2,
        });
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0].goal_key, "a");
        assert_eq!(scheduled[0].plan, RealizationPlan::RequireLocal);
    }

    #[test]
    fn scheduler_selection_keeps_goal_dedup_owned_by_scheduler() {
        let ready = vec![
            ReadyDerivationGoal {
                goal_key: "dup".to_string(),
                request: base_request(),
            },
            ReadyDerivationGoal {
                goal_key: "dup".to_string(),
                request: base_request(),
            },
        ];

        let scheduled = select_ready_realizations(&RemoteAllowedRealizationPolicy, &ready, RealizationJobCapacity {
            running_jobs: 0,
            max_jobs: 8,
        });
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0].plan, RealizationPlan::AllowRemote);
    }

    #[test]
    fn scheduler_selection_does_not_schedule_when_at_capacity() {
        let ready = vec![ReadyDerivationGoal {
            goal_key: "a".to_string(),
            request: base_request(),
        }];

        assert!(
            select_ready_realizations(&LocalOnlyRealizationPolicy, &ready, RealizationJobCapacity {
                running_jobs: 2,
                max_jobs: 2,
            },)
            .is_empty()
        );
    }

    #[test]
    fn local_only_policy_requires_local_realizer() {
        assert_eq!(LocalOnlyRealizationPolicy.plan_for(&base_request()), RealizationPlan::RequireLocal);
    }

    #[test]
    fn remote_allowed_policy_allows_remote_realizer() {
        assert_eq!(RemoteAllowedRealizationPolicy.plan_for(&base_request()), RealizationPlan::AllowRemote);
    }

    #[test]
    fn remote_goal_attachments_start_once_and_attach_duplicate_key() {
        let ready = vec![
            ReadyDerivationGoal {
                goal_key: "root-a".to_string(),
                request: base_request(),
            },
            ReadyDerivationGoal {
                goal_key: "root-b".to_string(),
                request: base_request(),
            },
        ];
        let attachments = plan_remote_goal_attachments(&DefaultRealizationKeyDeriver, &ready, &BTreeMap::new())
            .expect("remote goal attachment plans");

        assert_eq!(attachments.len(), ready.len());
        assert!(
            matches!(attachments[0], RemoteGoalAttachment::StartRemote { ref goal_key, .. } if goal_key == "root-a")
        );
        assert!(matches!(
            attachments[1],
            RemoteGoalAttachment::AttachToExisting { ref goal_key, ref owner_goal_key, .. }
                if goal_key == "root-b" && owner_goal_key == "root-a"
        ));
    }

    #[test]
    fn remote_goal_attachments_reject_unbounded_ready_sets() {
        let ready = vec![
            ReadyDerivationGoal {
                goal_key: "root".to_string(),
                request: base_request(),
            };
            MAX_READY_REMOTE_GOALS.saturating_add(1)
        ];
        let err = plan_remote_goal_attachments(&DefaultRealizationKeyDeriver, &ready, &BTreeMap::new())
            .expect_err("unbounded remote ready set rejected");

        assert!(matches!(err, RemoteScheduleError::TooManyReadyGoals {
            max: MAX_READY_REMOTE_GOALS
        }));
    }

    #[test]
    fn remote_build_service_adapter_dispatches_concrete_ready_goal() {
        #[derive(Clone)]
        struct RecordingRemoteRealizer(std::sync::Arc<std::sync::atomic::AtomicUsize>);

        #[async_trait::async_trait]
        impl DerivationRealizer for RecordingRemoteRealizer {
            fn profile(&self) -> RealizerProfileFacts {
                RealizerProfileFacts {
                    name: "remote".to_string(),
                    version: TEST_PROFILE_VERSION,
                    capabilities: vec!["remote-build".to_string()],
                    parameters: BTreeMap::new(),
                }
            }

            async fn realize(
                &self,
                request: snix_build::buildservice::BuildRequest,
            ) -> std::io::Result<snix_build::buildservice::BuildResult> {
                validate_remote_build_service_request(&request).unwrap();
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(build_result("remote"))
            }
        }

        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let service = RemoteBuildServiceAdapter::new(RecordingRemoteRealizer(calls.clone()));
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = rt.block_on(service.do_build(concrete_build_request())).unwrap();

        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.log.as_deref(), Some("remote"));
    }

    #[test]
    fn remote_build_service_rejects_raw_or_incomplete_requests() {
        let mut raw = concrete_build_request();
        raw.command_args.clear();
        let err = validate_remote_build_service_request(&raw).expect_err("raw frontend request rejected");
        assert!(matches!(
            err,
            RemoteBuildServiceDispatchError::Phase {
                phase: REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION,
                reason
            } if reason == "remote-build-service-raw-eval-request"
        ));

        let mut incomplete = concrete_build_request();
        incomplete.outputs.clear();
        let err = validate_remote_build_service_request(&incomplete).expect_err("output-less request rejected");
        assert!(matches!(
            err,
            RemoteBuildServiceDispatchError::Phase {
                phase: REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION,
                reason
            } if reason == "remote-build-service-outputs-empty"
        ));
    }

    #[test]
    fn remote_first_fallback_requires_explicit_policy() {
        #[derive(Clone)]
        struct FailingRemote;

        #[async_trait::async_trait]
        impl DerivationRealizer for FailingRemote {
            fn profile(&self) -> RealizerProfileFacts {
                RealizerProfileFacts {
                    name: "remote".to_string(),
                    version: TEST_PROFILE_VERSION,
                    capabilities: vec!["remote-build".to_string()],
                    parameters: BTreeMap::new(),
                }
            }

            async fn realize(
                &self,
                _request: snix_build::buildservice::BuildRequest,
            ) -> std::io::Result<snix_build::buildservice::BuildResult> {
                Err(std::io::Error::other("remote denied"))
            }
        }

        #[derive(Clone)]
        struct CountingLocal(std::sync::Arc<std::sync::atomic::AtomicUsize>);

        #[async_trait::async_trait]
        impl snix_build::buildservice::BuildService for CountingLocal {
            async fn do_build(
                &self,
                _request: snix_build::buildservice::BuildRequest,
            ) -> std::io::Result<snix_build::buildservice::BuildResult> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(build_result("local"))
            }
        }

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let denied_local_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let denied = RemoteFirstBuildService::new(
            FailingRemote,
            CountingLocal(denied_local_calls.clone()),
            RemoteBuildFallbackPolicy::Never,
        );
        let err = rt.block_on(denied.do_build(concrete_build_request())).unwrap_err();
        assert!(err.to_string().contains(REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH));
        assert_eq!(denied_local_calls.load(std::sync::atomic::Ordering::SeqCst), 0);

        let allowed_local_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let allowed = RemoteFirstBuildService::new(
            FailingRemote,
            CountingLocal(allowed_local_calls.clone()),
            RemoteBuildFallbackPolicy::OnRemoteFailure,
        );
        let result = rt.block_on(allowed.do_build(concrete_build_request())).unwrap();
        assert_eq!(allowed_local_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(result.log.as_deref(), Some("local"));
    }

    #[test]
    fn local_build_service_realizer_delegates_to_build_service() {
        #[derive(Clone)]
        struct RecordingBuildService(std::sync::Arc<std::sync::atomic::AtomicUsize>);

        #[async_trait::async_trait]
        impl snix_build::buildservice::BuildService for RecordingBuildService {
            async fn do_build(
                &self,
                request: snix_build::buildservice::BuildRequest,
            ) -> std::io::Result<snix_build::buildservice::BuildResult> {
                assert_eq!(request.command_args, vec!["/bin/true".to_string()]);
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(snix_build::buildservice::BuildResult {
                    outputs: vec![],
                    log: Some("local".to_string()),
                })
            }
        }

        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let realizer =
            LocalBuildServiceRealizer::new(RecordingBuildService(calls.clone()), base_request().realizer_profile);
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = rt
            .block_on(realizer.realize(snix_build::buildservice::BuildRequest {
                command_args: vec!["/bin/true".to_string()],
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.log.as_deref(), Some("local"));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn fake_remote_realizer_can_satisfy_contract_tests_without_local_service() {
        struct FakeRemoteRealizer;

        #[async_trait::async_trait]
        impl DerivationRealizer for FakeRemoteRealizer {
            fn profile(&self) -> RealizerProfileFacts {
                RealizerProfileFacts {
                    name: "fake-remote".to_string(),
                    version: 1,
                    capabilities: vec!["remote".to_string()],
                    parameters: BTreeMap::new(),
                }
            }

            async fn realize(
                &self,
                _request: snix_build::buildservice::BuildRequest,
            ) -> std::io::Result<snix_build::buildservice::BuildResult> {
                Ok(snix_build::buildservice::BuildResult {
                    outputs: vec![],
                    log: Some("remote".to_string()),
                })
            }
        }

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = rt.block_on(FakeRemoteRealizer.realize(Default::default())).unwrap();
        assert_eq!(result.log.as_deref(), Some("remote"));
        assert_eq!(FakeRemoteRealizer.profile().name, "fake-remote");
    }

    #[test]
    fn empty_resolver_chain_preserves_local_build_decision() {
        let request = ArtifactResolveRequest {
            key: key(&base_request()),
        };
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let mut resolvers: Vec<&mut dyn ArtifactResolver> = vec![];
            assert_eq!(
                resolve_before_dispatch(&mut resolvers, &request).await.unwrap(),
                RealizationDispatchDecision::BuildLocally
            );
        });
    }

    #[test]
    fn resolver_chain_uses_first_hit_before_local_build() {
        let request = ArtifactResolveRequest {
            key: key(&base_request()),
        };
        let artifact = verified_artifact();
        let mut miss = InMemoryArtifactAdapter::default();
        let mut hit = InMemoryArtifactAdapter::default();
        hit.insert_verified(request.key.clone(), artifact.clone());

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let mut resolvers: Vec<&mut dyn ArtifactResolver> = vec![&mut miss, &mut hit];
            assert_eq!(
                resolve_before_dispatch(&mut resolvers, &request).await.unwrap(),
                RealizationDispatchDecision::UseResolvedArtifact(artifact)
            );
        });
    }

    #[test]
    fn resolver_chain_skips_unavailable_resolvers() {
        struct UnavailableResolver;

        #[async_trait::async_trait]
        impl ArtifactResolver for UnavailableResolver {
            async fn resolve(
                &mut self,
                _request: &ArtifactResolveRequest,
            ) -> Result<ResolveOutcome, ArtifactAdapterError> {
                Ok(ResolveOutcome::Unavailable {
                    reason: "maintenance".to_string(),
                })
            }
        }

        let request = ArtifactResolveRequest {
            key: key(&base_request()),
        };
        let mut unavailable = UnavailableResolver;
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let mut resolvers: Vec<&mut dyn ArtifactResolver> = vec![&mut unavailable];
            assert_eq!(
                resolve_before_dispatch(&mut resolvers, &request).await.unwrap(),
                RealizationDispatchDecision::BuildLocally
            );
        });
    }

    #[test]
    fn equivalent_requests_have_stable_keys() {
        let request = base_request();
        let equivalent = base_request();

        assert_eq!(key(&request), key(&equivalent));
        assert!(key(&request).as_str().starts_with("crunch-realization-key-v1:"));
    }

    #[test]
    fn derivation_identity_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.derivation.identity = "/crunch/store/99999999999999999999999999999999-example.drv".to_string();

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn input_closure_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.input_closure[0].nar_hash =
            "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string();

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn platform_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.platform.system = "aarch64-linux".to_string();

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn toolchain_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.toolchains[0].store_path = "/crunch/store/66666666666666666666666666666666-gcc".to_string();

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn sandbox_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.sandbox.hermeticity = "practical".to_string();

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn environment_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.environment.insert("CC".to_string(), b"gcc".to_vec());

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn store_prefix_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.store.logical_prefix = "/nix/store".to_string();

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn realizer_profile_changes_key() {
        let original = base_request();
        let mut changed = base_request();
        changed.realizer_profile.parameters.insert("cpu".to_string(), "zen4".to_string());

        assert_ne!(key(&original), key(&changed));
    }

    #[test]
    fn realization_key_matches_legacy_serialization_bytes() {
        let request = base_request();
        let bytes = serde_json::to_vec(&request).expect("legacy request serialization");
        let digest = blake3::hash(&bytes);
        let legacy = format!("crunch-realization-key-v1:{}", data_encoding::HEXLOWER.encode(digest.as_bytes()));
        assert_eq!(key(&request).as_str(), legacy);
        assert!(legacy.starts_with("crunch-realization-key-v1:"));
    }

    #[test]
    fn input_closure_order_is_validated() {
        let mut changed = base_request();
        changed.input_closure.swap(0, 1);

        assert!(matches!(
            DefaultRealizationKeyDeriver.derive_key(&changed),
            Err(RealizationKeyError::NotSortedUnique {
                field: "input_closure.store_path"
            })
        ));
    }
}
