//! Provider-neutral distributed build seams.
//!
//! This module is intentionally pure for the first distributed-build slice:
//! it defines realization-key inputs and deterministic key derivation without
//! contacting stores, schedulers, providers, or network services.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

mod external_batch;
mod remote_attempt_log;
mod remote_failure_debug;
mod remote_resources;
mod remote_telemetry;
mod remote_transfer;
mod resource_policy;
mod resource_quota_store;
pub use external_batch::*;
pub use remote_attempt_log::*;
pub use remote_failure_debug::*;
pub use remote_resources::*;
pub use remote_telemetry::*;
pub use remote_transfer::*;
pub use resource_policy::*;
pub use resource_quota_store::*;

use crate::scheduling::ContentLocalityClass;
use crate::scheduling::EligiblePreferenceFacts;
use crate::scheduling::HardEligibilityFacts;
use crate::scheduling::IneligibleReason;
use crate::scheduling::ResourceFitClass;
use crate::scheduling::TransferCostClass;
use crate::scheduling::normalize_eligible_preference;

const REALIZATION_KEY_SCHEMA: &str = "crunch-realization-key-v1";
const MAX_READY_REMOTE_GOALS: usize = 4096;
const REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION: &str = "request-validation";
const REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH: &str = "remote-dispatch";
const OPERATOR_SECRET_MARKERS: [&str; 5] = ["bearer ", "token=", "authorization", "secret", "password"];

/// Stable provider-neutral key for a derivation realization request.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RealizationKey(String);

impl RealizationKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Derives deterministic realization keys from normalized Crunch facts.
pub trait RealizationKeyDeriver {
    fn derive_key(&self, request: &RealizationKeyRequest) -> Result<RealizationKey, RealizationKeyError>;
}

/// Default BLAKE3-based provider-neutral key deriver.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRealizationKeyDeriver;

impl RealizationKeyDeriver for DefaultRealizationKeyDeriver {
    fn derive_key(&self, request: &RealizationKeyRequest) -> Result<RealizationKey, RealizationKeyError> {
        request.validate()?;
        let bytes = serde_json::to_vec(request).map_err(|source| RealizationKeyError::Serialize { source })?;
        let digest = blake3::hash(&bytes);
        Ok(RealizationKey(format!(
            "{REALIZATION_KEY_SCHEMA}:{}",
            data_encoding::HEXLOWER.encode(digest.as_bytes())
        )))
    }
}

/// Normalized facts that can affect a derivation realization output or the
/// admissible execution environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizationKeyRequest {
    pub derivation: DerivationKeyFacts,
    pub input_closure: Vec<InputClosureFact>,
    pub platform: PlatformFacts,
    pub toolchains: Vec<ToolchainFact>,
    pub sandbox: SandboxFacts,
    pub environment: BTreeMap<String, Vec<u8>>,
    pub store: StorePrefixFacts,
    pub realizer_profile: RealizerProfileFacts,
}

impl RealizationKeyRequest {
    fn validate(&self) -> Result<(), RealizationKeyError> {
        validate_non_empty(NonEmptyField {
            field: "derivation.identity",
            value: &self.derivation.identity,
        })?;
        validate_non_empty(NonEmptyField {
            field: "derivation.builder",
            value: &self.derivation.builder,
        })?;
        validate_non_empty(NonEmptyField {
            field: "platform.system",
            value: &self.platform.system,
        })?;
        validate_non_empty(NonEmptyField {
            field: "store.logical_prefix",
            value: &self.store.logical_prefix,
        })?;
        validate_non_empty(NonEmptyField {
            field: "store.output_prefix",
            value: &self.store.output_prefix,
        })?;
        validate_non_empty(NonEmptyField {
            field: "sandbox.hermeticity",
            value: &self.sandbox.hermeticity,
        })?;
        validate_non_empty(NonEmptyField {
            field: "realizer_profile.name",
            value: &self.realizer_profile.name,
        })?;
        validate_sorted_unique_by(&self.input_closure, |fact| fact.store_path.as_str(), "input_closure.store_path")?;
        validate_sorted_unique_by(&self.toolchains, |fact| fact.name.as_str(), "toolchains.name")?;
        validate_sorted_unique_by(
            &self.realizer_profile.capabilities,
            String::as_str,
            "realizer_profile.capabilities",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationKeyFacts {
    pub identity: String,
    pub builder: String,
    pub args: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputClosureFact {
    pub store_path: String,
    pub nar_hash: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformFacts {
    pub system: String,
    pub cpu: String,
    pub os: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainFact {
    pub name: String,
    pub store_path: String,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxFacts {
    pub hermeticity: String,
    pub network_allowed: bool,
    pub fixed_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorePrefixFacts {
    pub logical_prefix: String,
    pub output_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizerProfileFacts {
    pub name: String,
    pub version: u32,
    pub capabilities: Vec<String>,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRealizationArtifact {
    pub outputs: BTreeMap<String, snix_store::path_info::PathInfo>,
    pub substitutions: BTreeMap<String, crunch_store::OutputSubstitutionReport>,
}

impl VerifiedRealizationArtifact {
    pub fn new(
        outputs: BTreeMap<String, snix_store::path_info::PathInfo>,
        substitutions: BTreeMap<String, crunch_store::OutputSubstitutionReport>,
    ) -> Result<Self, ArtifactAdapterError> {
        if outputs.is_empty() {
            return Err(ArtifactAdapterError::EmptyArtifact);
        }
        for output_name in substitutions.keys() {
            if !outputs.contains_key(output_name) {
                return Err(ArtifactAdapterError::UnknownSubstitutionOutput {
                    output: output_name.clone(),
                });
            }
        }
        Ok(Self { outputs, substitutions })
    }
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

impl RealizationKey {
    pub fn short(&self) -> &str {
        self.0.get(..12).unwrap_or(&self.0)
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

#[async_trait::async_trait]
pub trait DerivationRealizer: Send + Sync {
    fn profile(&self) -> RealizerProfileFacts;

    async fn realize(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult>;
}

#[derive(Debug, Clone)]
pub struct LocalBuildServiceRealizer<S> {
    service: S,
    profile: RealizerProfileFacts,
}

impl<S> LocalBuildServiceRealizer<S> {
    pub fn new(service: S, profile: RealizerProfileFacts) -> Self {
        Self { service, profile }
    }
}

#[async_trait::async_trait]
impl<S> DerivationRealizer for LocalBuildServiceRealizer<S>
where S: snix_build::buildservice::BuildService + Send + Sync
{
    fn profile(&self) -> RealizerProfileFacts {
        self.profile.clone()
    }

    async fn realize(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult> {
        self.service.do_build(request).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteBuildFallbackPolicy {
    Never,
    OnRemoteFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RemoteBuildServiceDispatchError {
    #[error("remote build service {phase} failed: {reason}")]
    Phase { phase: &'static str, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFailureDecision {
    ReturnFailure(RemoteBuildServiceDispatchError),
    FallbackToLocal { phase: &'static str, reason: String },
}

pub fn validate_remote_build_service_request(
    request: &snix_build::buildservice::BuildRequest,
) -> Result<(), RemoteBuildServiceDispatchError> {
    crunch_remote_core::provider::admit_build_service_request(crunch_remote_core::provider::BuildServiceRequestFacts {
        command_arg_count: request.command_args.len(),
        output_count: request.outputs.len(),
    })
    .map_err(|reason| RemoteBuildServiceDispatchError::Phase {
        phase: REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION,
        reason: reason.as_str().to_string(),
    })
}

pub fn classify_remote_build_service_failure(
    policy: RemoteBuildFallbackPolicy,
    phase: &'static str,
    reason: impl Into<String>,
) -> RemoteFailureDecision {
    let reason = reason.into();
    let core_policy = match policy {
        RemoteBuildFallbackPolicy::Never => crunch_remote_core::provider::DispatchFallbackPolicy::Never,
        RemoteBuildFallbackPolicy::OnRemoteFailure => {
            crunch_remote_core::provider::DispatchFallbackPolicy::OnRemoteFailure
        }
    };
    match crunch_remote_core::provider::classify_dispatch_failure(core_policy) {
        crunch_remote_core::provider::DispatchFailureDecision::ReturnFailure => {
            RemoteFailureDecision::ReturnFailure(RemoteBuildServiceDispatchError::Phase { phase, reason })
        }
        crunch_remote_core::provider::DispatchFailureDecision::FallbackToLocal => {
            RemoteFailureDecision::FallbackToLocal { phase, reason }
        }
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

#[derive(Debug, Clone)]
pub struct RemoteBuildServiceAdapter<R> {
    remote: R,
}

impl<R> RemoteBuildServiceAdapter<R> {
    pub fn new(remote: R) -> Self {
        Self { remote }
    }
}

#[async_trait::async_trait]
impl<R> snix_build::buildservice::BuildService for RemoteBuildServiceAdapter<R>
where R: DerivationRealizer
{
    async fn do_build(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult> {
        validate_remote_build_service_request(&request).map_err(remote_dispatch_io_error)?;
        self.remote.realize(request).await.map_err(|source| {
            remote_dispatch_io_error(RemoteBuildServiceDispatchError::Phase {
                phase: REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH,
                reason: source.to_string(),
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct RemoteFirstBuildService<R, L> {
    remote: R,
    local: L,
    fallback_policy: RemoteBuildFallbackPolicy,
}

impl<R, L> RemoteFirstBuildService<R, L> {
    pub fn new(remote: R, local: L, fallback_policy: RemoteBuildFallbackPolicy) -> Self {
        Self {
            remote,
            local,
            fallback_policy,
        }
    }
}

#[async_trait::async_trait]
impl<R, L> snix_build::buildservice::BuildService for RemoteFirstBuildService<R, L>
where
    R: DerivationRealizer,
    L: snix_build::buildservice::BuildService,
{
    async fn do_build(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult> {
        validate_remote_build_service_request(&request).map_err(remote_dispatch_io_error)?;
        match self.remote.realize(request.clone()).await {
            Ok(result) => Ok(result),
            Err(source) => match classify_remote_build_service_failure(
                self.fallback_policy,
                REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH,
                source.to_string(),
            ) {
                RemoteFailureDecision::ReturnFailure(error) => Err(remote_dispatch_io_error(error)),
                RemoteFailureDecision::FallbackToLocal { .. } => self.local.do_build(request).await,
            },
        }
    }
}

fn remote_dispatch_io_error(error: RemoteBuildServiceDispatchError) -> std::io::Error {
    std::io::Error::other(error.to_string())
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
        validate_sorted_unique_by(&self.outputs, |output| output.output_name.as_str(), "receipt.outputs")
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

#[derive(Debug, thiserror::Error)]
pub enum RealizationKeyError {
    #[error("realization key field {field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("realization key collection {field} must be sorted and unique")]
    NotSortedUnique { field: &'static str },
    #[error("serializing realization key request: {source}")]
    Serialize { source: serde_json::Error },
}

struct NonEmptyField<'a> {
    field: &'static str,
    value: &'a str,
}

fn validate_non_empty(input: NonEmptyField<'_>) -> Result<(), RealizationKeyError> {
    if input.value.is_empty() {
        return Err(RealizationKeyError::EmptyField { field: input.field });
    }
    Ok(())
}

fn validate_sorted_unique_by<'a, T, F>(items: &'a [T], key: F, field: &'static str) -> Result<(), RealizationKeyError>
where F: Fn(&'a T) -> &'a str {
    let mut previous: Option<&str> = None;
    for item in items {
        let current = key(item);
        if current.is_empty() || previous.is_some_and(|prev| prev >= current) {
            return Err(RealizationKeyError::NotSortedUnique { field });
        }
        previous = Some(current);
    }
    Ok(())
}

/// One admitted pool endpoint with its own execution service.
#[derive(Debug, Clone)]
pub struct PoolEndpointService<S> {
    endpoint_id: String,
    service: S,
}

impl<S> PoolEndpointService<S> {
    pub fn new(endpoint_id: impl Into<String>, service: S) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            service,
        }
    }

    pub fn endpoint_id(&self) -> &str {
        &self.endpoint_id
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PoolSelectionError {
    #[error("pool has no admitted endpoints")]
    NoEndpoints,
    #[error("pool endpoint id is empty or duplicated")]
    InvalidEndpoint,
    #[error("no admitted endpoint fits this goal")]
    NoFit,
    #[error("selector chose an endpoint outside the admitted pool")]
    OutOfRange,
}

/// Executes one prepared goal on one admitted endpoint. A pool endpoint is not
/// necessarily a Snix `BuildService`: the remote client needs the prepared
/// payload, which a bare `BuildRequest` does not carry.
#[async_trait::async_trait]
pub trait PoolGoalExecutor: Send + Sync {
    async fn execute(&self, goal: PreparedPoolGoal) -> std::io::Result<snix_build::buildservice::BuildResult>;
}

/// Adapter for endpoints that really are Snix build services. It executes the
/// goal's concrete request and ignores the extra resolved facts.
#[derive(Debug, Clone)]
pub struct BuildServiceExecutor<S> {
    service: S,
}

impl<S> BuildServiceExecutor<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl<S> PoolGoalExecutor for BuildServiceExecutor<S>
where S: snix_build::buildservice::BuildService
{
    async fn execute(&self, goal: PreparedPoolGoal) -> std::io::Result<snix_build::buildservice::BuildResult> {
        self.service.do_build(goal.request).await
    }
}

/// Executes each prepared goal on exactly one admitted pool endpoint. It owns
/// no placement policy, reservation, or fallback: the injected selector decides
/// from resolved facts, and the selected endpoint's service performs the build.
pub struct PoolDispatch {
    endpoints: Vec<PoolEndpointService<std::sync::Arc<dyn PoolGoalExecutor>>>,
    endpoint_ids: Vec<String>,
    selector: std::sync::Arc<dyn PoolGoalSelector>,
}

impl std::fmt::Debug for PoolDispatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PoolDispatch")
            .field("endpoint_ids", &self.endpoint_ids)
            .finish_non_exhaustive()
    }
}

impl PoolDispatch {
    pub fn new(
        endpoints: Vec<PoolEndpointService<std::sync::Arc<dyn PoolGoalExecutor>>>,
        selector: std::sync::Arc<dyn PoolGoalSelector>,
    ) -> Result<Self, PoolSelectionError> {
        if endpoints.is_empty() {
            return Err(PoolSelectionError::NoEndpoints);
        }
        let endpoint_ids = endpoints.iter().map(|endpoint| endpoint.endpoint_id.clone()).collect::<Vec<_>>();
        let mut seen = std::collections::BTreeSet::new();
        let is_valid = endpoint_ids.iter().all(|id| !id.is_empty() && seen.insert(id.clone()));
        if !is_valid {
            return Err(PoolSelectionError::InvalidEndpoint);
        }
        debug_assert_eq!(endpoint_ids.len(), endpoints.len());
        debug_assert!(endpoint_ids.iter().all(|id| !id.is_empty()));
        Ok(Self {
            endpoints,
            endpoint_ids,
            selector,
        })
    }

    pub fn endpoint_ids(&self) -> &[String] {
        &self.endpoint_ids
    }

    /// Validate, select from resolved facts, and execute on that endpoint only.
    pub async fn dispatch(&self, goal: PreparedPoolGoal) -> std::io::Result<snix_build::buildservice::BuildResult> {
        validate_remote_build_service_request(&goal.request).map_err(remote_dispatch_io_error)?;
        let index = self
            .selector
            .select(&goal, &self.endpoint_ids)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("remote-pool:{error}")))?;
        let endpoint = self.endpoints.get(index).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "remote-pool:selector-out-of-range")
        })?;
        endpoint.service.execute(goal).await
    }
}

/// One derivation in the resolved closure: exact path, derivation-modulo hash,
/// and canonical derivation bytes. The Worker already holds these in its
/// registry; a remote executor sends them because a bare request cannot express
/// the closure.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolClosureEntry {
    pub drv_path: String,
    pub hash_derivation_modulo: String,
    pub derivation_json: String,
}

/// Maximum derivations one goal closure may carry.
pub const MAX_POOL_GOAL_CLOSURE_ENTRIES: usize = 4_096;
/// Maximum combined closure derivation bytes.
pub const MAX_POOL_GOAL_CLOSURE_BYTES: usize = 67_108_864;

/// Facts the Worker resolved for one ready native goal. A bare
/// `BuildRequest` cannot express pool placement requirements, so the executor
/// seam must carry these explicitly rather than infer them from the request.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPoolGoal {
    pub request: snix_build::buildservice::BuildRequest,
    /// Canonical semantic action identity the Worker already resolved.
    pub action_ref: String,
    /// Absolute logical store path of the concrete derivation.
    pub drv_path: String,
    /// The concrete derivation serialized canonically. A remote executor sends
    /// this because a `BuildRequest` cannot express it and the Nickel closure is
    /// not retained after conversion.
    pub derivation_json: String,
    /// Resolved derivation closure (root plus transitive inputs), bounded.
    pub closure: Vec<PoolClosureEntry>,
    pub system: String,
    pub required_features: Vec<String>,
    pub required_sandbox_mode: String,
    pub required_network_mode: String,
    pub semantic_accelerator_classes: Vec<String>,
    /// BLAKE3 of `action_ref`; binds the semantic requirement, never capacity.
    pub placement_key_blake3: String,
    pub requested: crunch_remote_core::resource::Capacity,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PreparedPoolGoalError {
    #[error("prepared pool goal system is empty or too long")]
    InvalidSystem,
    #[error("prepared pool goal sandbox or network mode is empty")]
    InvalidMode,
    #[error("prepared pool goal action identity is empty or too long")]
    InvalidActionIdentity,
    #[error("prepared pool goal derivation path is empty or too long")]
    InvalidDerivationPath,
    #[error("prepared pool goal derivation payload is empty or too large")]
    InvalidDerivationPayload,
    #[error("prepared pool goal closure is malformed or too large")]
    InvalidClosure,
    #[error("prepared pool goal declares too many features or classes")]
    TooManyCapabilities,
    #[error("prepared pool goal capability name is empty or too long")]
    InvalidCapabilityName,
}

pub const MAX_POOL_GOAL_CAPABILITIES: usize = 32;
pub const MAX_POOL_GOAL_CAPABILITY_BYTES: usize = 128;
pub const MAX_POOL_GOAL_SYSTEM_BYTES: usize = 128;
pub const MAX_POOL_GOAL_ACTION_REF_BYTES: usize = 512;
/// Matches the remote build payload bound.
pub const MAX_POOL_GOAL_DERIVATION_BYTES: usize = 1_048_576;
pub const MAX_POOL_GOAL_DRV_PATH_BYTES: usize = 512;

impl PreparedPoolGoal {
    pub fn new(facts: PreparedPoolGoalFacts) -> Result<Self, PreparedPoolGoalError> {
        let has_system = !facts.system.is_empty() && facts.system.len() <= MAX_POOL_GOAL_SYSTEM_BYTES;
        if !has_system {
            return Err(PreparedPoolGoalError::InvalidSystem);
        }
        let has_modes = !facts.required_sandbox_mode.is_empty() && !facts.required_network_mode.is_empty();
        if !has_modes {
            return Err(PreparedPoolGoalError::InvalidMode);
        }
        let has_action = !facts.action_ref.is_empty() && facts.action_ref.len() <= MAX_POOL_GOAL_ACTION_REF_BYTES;
        if !has_action {
            return Err(PreparedPoolGoalError::InvalidActionIdentity);
        }
        let has_drv_path = !facts.drv_path.is_empty()
            && facts.drv_path.starts_with('/')
            && facts.drv_path.len() <= MAX_POOL_GOAL_DRV_PATH_BYTES;
        if !has_drv_path {
            return Err(PreparedPoolGoalError::InvalidDerivationPath);
        }
        let has_derivation =
            !facts.derivation_json.is_empty() && facts.derivation_json.len() <= MAX_POOL_GOAL_DERIVATION_BYTES;
        if !has_derivation {
            return Err(PreparedPoolGoalError::InvalidDerivationPayload);
        }
        validate_pool_closure(&facts.closure)?;
        let placement_key_blake3 = blake3::hash(facts.action_ref.as_bytes()).to_hex().to_string();
        let capability_count = facts.required_features.len() + facts.semantic_accelerator_classes.len();
        if capability_count > MAX_POOL_GOAL_CAPABILITIES {
            return Err(PreparedPoolGoalError::TooManyCapabilities);
        }
        let names = facts.required_features.iter().chain(facts.semantic_accelerator_classes.iter());
        let has_valid_names =
            names.clone().all(|name| !name.is_empty() && name.len() <= MAX_POOL_GOAL_CAPABILITY_BYTES);
        if !has_valid_names {
            return Err(PreparedPoolGoalError::InvalidCapabilityName);
        }
        debug_assert!(has_system && has_modes && has_action && has_drv_path && has_derivation);
        debug_assert!(capability_count <= MAX_POOL_GOAL_CAPABILITIES);
        Ok(Self {
            request: facts.request,
            action_ref: facts.action_ref,
            drv_path: facts.drv_path,
            derivation_json: facts.derivation_json,
            closure: facts.closure,
            system: facts.system,
            required_features: facts.required_features,
            required_sandbox_mode: facts.required_sandbox_mode,
            required_network_mode: facts.required_network_mode,
            semantic_accelerator_classes: facts.semantic_accelerator_classes,
            placement_key_blake3,
            requested: facts.requested,
        })
    }
}

fn validate_pool_closure(closure: &[PoolClosureEntry]) -> Result<(), PreparedPoolGoalError> {
    if closure.len() > MAX_POOL_GOAL_CLOSURE_ENTRIES {
        return Err(PreparedPoolGoalError::InvalidClosure);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut total_bytes = 0_usize;
    for entry in closure {
        let has_path = !entry.drv_path.is_empty()
            && entry.drv_path.starts_with('/')
            && entry.drv_path.len() <= MAX_POOL_GOAL_DRV_PATH_BYTES;
        let has_hdm = entry.hash_derivation_modulo.len() == 64
            && entry
                .hash_derivation_modulo
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        let has_bytes =
            !entry.derivation_json.is_empty() && entry.derivation_json.len() <= MAX_POOL_GOAL_DERIVATION_BYTES;
        if !has_path || !has_hdm || !has_bytes || !seen.insert(entry.drv_path.as_str()) {
            return Err(PreparedPoolGoalError::InvalidClosure);
        }
        total_bytes = total_bytes.saturating_add(entry.derivation_json.len());
    }
    if total_bytes > MAX_POOL_GOAL_CLOSURE_BYTES {
        return Err(PreparedPoolGoalError::InvalidClosure);
    }
    debug_assert!(closure.len() <= MAX_POOL_GOAL_CLOSURE_ENTRIES);
    debug_assert!(total_bytes <= MAX_POOL_GOAL_CLOSURE_BYTES);
    Ok(())
}

/// Owned inputs to `PreparedPoolGoal::new`; kept separate so callers can build
/// the facts without moving the request twice.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPoolGoalFacts {
    pub request: snix_build::buildservice::BuildRequest,
    pub action_ref: String,
    pub drv_path: String,
    pub derivation_json: String,
    pub closure: Vec<PoolClosureEntry>,
    pub system: String,
    pub required_features: Vec<String>,
    pub required_sandbox_mode: String,
    pub required_network_mode: String,
    pub semantic_accelerator_classes: Vec<String>,
    pub requested: crunch_remote_core::resource::Capacity,
}

/// Chooses the endpoint for one prepared goal. The selector receives the
/// resolved facts, never a bare `BuildRequest`.
pub trait PoolGoalSelector: Send + Sync {
    fn select(&self, goal: &PreparedPoolGoal, endpoint_ids: &[String]) -> Result<usize, PoolSelectionError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_ne;
    use snix_build::buildservice::BuildService;

    use super::*;

    const TEST_PROFILE_VERSION: u32 = 1;
    const TEST_HARD_BLOCKER_CASE_COUNT: usize = 6;

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
        let store_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("artifact", [7u8; 20]).unwrap();
        let path_info = snix_store::path_info::PathInfo {
            store_path,
            node: snix_castore::Node::Symlink {
                target: "/crunch/store/source".try_into().unwrap(),
            },
            references: vec![],
            nar_sha256: [8u8; 32],
            nar_size: 128,
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        VerifiedRealizationArtifact::new(BTreeMap::from([("out".to_string(), path_info)]), BTreeMap::new()).unwrap()
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
        assert!(matches!(
            VerifiedRealizationArtifact::new(BTreeMap::new(), BTreeMap::new()),
            Err(ArtifactAdapterError::EmptyArtifact)
        ));
    }

    #[test]
    fn verified_artifact_rejects_substitution_for_unknown_output() {
        let err = VerifiedRealizationArtifact::new(
            verified_artifact().outputs,
            BTreeMap::from([("missing".to_string(), crunch_store::OutputSubstitutionReport {
                mode: crunch_store::OutputSubstitutionMode::Full,
                transferred_bytes: 0,
                reused_bytes: 0,
                metadata_reused: false,
                fallback_reason: None,
            })]),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ArtifactAdapterError::UnknownSubstitutionOutput { output } if output == "missing"
        ));
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
        let key = key(&base_request());
        let mut expected = key.clone();
        expected.0 = "different".to_string();
        assert_eq!(
            remote_candidate(key).verify_for_persistence(&expected, &["out".to_string()]).unwrap_err(),
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

    struct FakePoolService {
        label: &'static str,
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl snix_build::buildservice::BuildService for FakePoolService {
        async fn do_build(
            &self,
            _request: snix_build::buildservice::BuildRequest,
        ) -> std::io::Result<snix_build::buildservice::BuildResult> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(build_result(self.label))
        }
    }

    struct SystemSelector {
        desired: &'static str,
    }

    impl PoolGoalSelector for SystemSelector {
        fn select(&self, goal: &PreparedPoolGoal, endpoint_ids: &[String]) -> Result<usize, PoolSelectionError> {
            if goal.system != self.desired {
                return Err(PoolSelectionError::NoFit);
            }
            endpoint_ids.iter().position(|id| id == "worker-b").ok_or(PoolSelectionError::NoFit)
        }
    }

    struct OutOfRangeSelector;

    impl PoolGoalSelector for OutOfRangeSelector {
        fn select(&self, _goal: &PreparedPoolGoal, _endpoint_ids: &[String]) -> Result<usize, PoolSelectionError> {
            Ok(9)
        }
    }

    fn pool_dispatch(
        selector: std::sync::Arc<dyn PoolGoalSelector>,
    ) -> (
        PoolDispatch,
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) {
        let first = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let second = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let dispatch = PoolDispatch::new(
            vec![
                PoolEndpointService::new(
                    "worker-a",
                    std::sync::Arc::new(BuildServiceExecutor::new(FakePoolService {
                        label: "a",
                        calls: first.clone(),
                    })) as std::sync::Arc<dyn PoolGoalExecutor>,
                ),
                PoolEndpointService::new(
                    "worker-b",
                    std::sync::Arc::new(BuildServiceExecutor::new(FakePoolService {
                        label: "b",
                        calls: second.clone(),
                    })) as std::sync::Arc<dyn PoolGoalExecutor>,
                ),
            ],
            selector,
        )
        .expect("fixture pool dispatch");
        (dispatch, first, second)
    }

    fn goal_with_system(system: &str) -> PreparedPoolGoal {
        let mut facts = prepared_pool_goal_facts();
        facts.system = system.to_string();
        PreparedPoolGoal::new(facts).expect("prepared goal")
    }

    #[test]
    fn pool_dispatch_executes_only_on_the_selected_endpoint() {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (dispatch, first, second) = pool_dispatch(std::sync::Arc::new(SystemSelector {
            desired: "x86_64-linux",
        }));
        assert_eq!(dispatch.endpoint_ids(), &[String::from("worker-a"), String::from("worker-b")]);
        let result = runtime.block_on(dispatch.dispatch(goal_with_system("x86_64-linux"))).unwrap();
        assert_eq!(result.log.as_deref(), Some("b"));
        assert_eq!(first.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(second.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn pool_dispatch_denies_unplaceable_out_of_range_and_invalid_pools() {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (no_fit, first, second) = pool_dispatch(std::sync::Arc::new(SystemSelector {
            desired: "aarch64-linux",
        }));
        let error = runtime.block_on(no_fit.dispatch(goal_with_system("x86_64-linux"))).unwrap_err();
        assert!(error.to_string().contains("remote-pool:no admitted endpoint fits"));
        assert_eq!(first.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(second.load(std::sync::atomic::Ordering::SeqCst), 0);

        let (out_of_range, _, _) = pool_dispatch(std::sync::Arc::new(OutOfRangeSelector));
        let error = runtime.block_on(out_of_range.dispatch(goal_with_system("x86_64-linux"))).unwrap_err();
        assert!(error.to_string().contains("selector-out-of-range"));

        let empty = PoolDispatch::new(Vec::new(), std::sync::Arc::new(OutOfRangeSelector));
        assert_eq!(empty.err(), Some(PoolSelectionError::NoEndpoints));

        let duplicate = PoolDispatch::new(
            vec![
                PoolEndpointService::new(
                    "worker-a",
                    std::sync::Arc::new(BuildServiceExecutor::new(FakePoolService {
                        label: "a",
                        calls: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
                    })) as std::sync::Arc<dyn PoolGoalExecutor>,
                ),
                PoolEndpointService::new(
                    "worker-a",
                    std::sync::Arc::new(BuildServiceExecutor::new(FakePoolService {
                        label: "b",
                        calls: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
                    })) as std::sync::Arc<dyn PoolGoalExecutor>,
                ),
            ],
            std::sync::Arc::new(OutOfRangeSelector),
        );
        assert_eq!(duplicate.err(), Some(PoolSelectionError::InvalidEndpoint));
    }

    #[test]
    fn pool_dispatch_validates_the_request_before_selecting() {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (dispatch, first, second) = pool_dispatch(std::sync::Arc::new(SystemSelector {
            desired: "x86_64-linux",
        }));
        let mut facts = prepared_pool_goal_facts();
        facts.request.command_args.clear();
        let goal = PreparedPoolGoal::new(facts).expect("prepared goal");
        let error = runtime.block_on(dispatch.dispatch(goal)).unwrap_err();
        assert!(error.to_string().contains("remote"));
        assert_eq!(first.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(second.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    fn prepared_pool_goal_facts() -> PreparedPoolGoalFacts {
        PreparedPoolGoalFacts {
            request: concrete_build_request(),
            action_ref: "action:fixture".to_string(),
            drv_path: "/mantle/store/00000000000000000000000000000000-fixture.drv".to_string(),
            derivation_json: "{\"name\":\"fixture\"}".to_string(),
            closure: Vec::new(),
            system: "x86_64-linux".to_string(),
            required_features: vec!["native-compile".to_string()],
            required_sandbox_mode: "practical".to_string(),
            required_network_mode: "none".to_string(),
            semantic_accelerator_classes: Vec::new(),
            requested: crunch_remote_core::resource::Capacity {
                cpu_units: 1,
                memory_bytes: 1_024,
                scratch_bytes: 1_024,
            },
        }
    }

    #[test]
    fn prepared_goal_requires_resolved_placement_facts() {
        let mut facts = prepared_pool_goal_facts();
        facts.system = String::new();
        assert_eq!(PreparedPoolGoal::new(facts.clone()).err(), Some(PreparedPoolGoalError::InvalidSystem));

        facts.system = "x86_64-linux".to_string();
        facts.required_network_mode = String::new();
        assert_eq!(PreparedPoolGoal::new(facts.clone()).err(), Some(PreparedPoolGoalError::InvalidMode));

        facts.required_network_mode = "none".to_string();
        facts.action_ref = String::new();
        assert_eq!(PreparedPoolGoal::new(facts.clone()).err(), Some(PreparedPoolGoalError::InvalidActionIdentity));

        facts.action_ref = "action:fixture".to_string();
        facts.derivation_json = String::new();
        assert_eq!(PreparedPoolGoal::new(facts.clone()).err(), Some(PreparedPoolGoalError::InvalidDerivationPayload));

        facts.derivation_json = "{\"name\":\"fixture\"}".to_string();
        facts.required_features = vec![String::new()];
        assert_eq!(PreparedPoolGoal::new(facts).err(), Some(PreparedPoolGoalError::InvalidCapabilityName));

        let goal = PreparedPoolGoal::new(prepared_pool_goal_facts()).expect("prepared goal");
        assert_eq!(goal.system, "x86_64-linux");
        assert_eq!(goal.requested.cpu_units, 1);
        assert_eq!(goal.placement_key_blake3, blake3::hash(b"action:fixture").to_hex().to_string());
    }

    #[test]
    fn goal_selector_receives_prepared_facts_not_the_bare_request() {
        struct SystemSelector;

        impl PoolGoalSelector for SystemSelector {
            fn select(&self, goal: &PreparedPoolGoal, endpoint_ids: &[String]) -> Result<usize, PoolSelectionError> {
                if goal.system != "x86_64-linux" || goal.required_features != ["native-compile".to_string()] {
                    return Err(PoolSelectionError::NoFit);
                }
                endpoint_ids.iter().position(|id| id == "worker-b").ok_or(PoolSelectionError::NoFit)
            }
        }

        let goal = PreparedPoolGoal::new(prepared_pool_goal_facts()).expect("prepared goal");
        let endpoints = vec![String::from("worker-a"), String::from("worker-b")];
        assert_eq!(SystemSelector.select(&goal, &endpoints), Ok(1));
        assert_eq!(SystemSelector.select(&goal, &[String::from("worker-a")]), Err(PoolSelectionError::NoFit));
    }
}
