//! Provider-neutral distributed build seams.
//!
//! This module is intentionally pure for the first distributed-build slice:
//! it defines realization-key inputs and deterministic key derivation without
//! contacting stores, schedulers, providers, or network services.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

const REALIZATION_KEY_SCHEMA: &str = "crunch-realization-key-v1";

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
        validate_non_empty("derivation.identity", &self.derivation.identity)?;
        validate_non_empty("derivation.builder", &self.derivation.builder)?;
        validate_non_empty("platform.system", &self.platform.system)?;
        validate_non_empty("store.logical_prefix", &self.store.logical_prefix)?;
        validate_non_empty("store.output_prefix", &self.store.output_prefix)?;
        validate_non_empty("sandbox.hermeticity", &self.sandbox.hermeticity)?;
        validate_non_empty("realizer_profile.name", &self.realizer_profile.name)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRealization {
    pub goal_key: String,
    pub plan: RealizationPlan,
}

pub fn select_ready_realizations<P>(
    policy: &P,
    ready: &[ReadyDerivationGoal],
    running_jobs: usize,
    max_jobs: usize,
) -> Vec<ScheduledRealization>
where
    P: RealizationPolicy,
{
    let available_slots = max_jobs.saturating_sub(running_jobs);
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

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RealizationKeyError> {
    if value.is_empty() {
        return Err(RealizationKeyError::EmptyField { field });
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_ne;

    use super::*;

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
                fallback_reason: None,
            })]),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ArtifactAdapterError::UnknownSubstitutionOutput { output } if output == "missing"
        ));
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

        let scheduled = select_ready_realizations(&LocalOnlyRealizationPolicy, &ready, 1, 2);
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

        let scheduled = select_ready_realizations(&RemoteAllowedRealizationPolicy, &ready, 0, 8);
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0].plan, RealizationPlan::AllowRemote);
    }

    #[test]
    fn scheduler_selection_does_not_schedule_when_at_capacity() {
        let ready = vec![ReadyDerivationGoal {
            goal_key: "a".to_string(),
            request: base_request(),
        }];

        assert!(select_ready_realizations(&LocalOnlyRealizationPolicy, &ready, 2, 2).is_empty());
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
}
