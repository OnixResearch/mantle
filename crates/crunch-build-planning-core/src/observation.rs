use alloc::string::String;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", content = "value", rename_all = "kebab-case")]
pub enum ObservedFact<T> {
    Present(T),
    Missing,
}

impl<T> ObservedFact<T> {
    #[must_use]
    pub const fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Present(value) => Some(value),
            Self::Missing => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationKind {
    LocalOutput,
    SourceReadiness,
    Substituter,
    Archive,
    RemoteCandidate,
    Doctor,
    Platform,
    Trust,
    Network,
    Executor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalOutputObservation {
    pub present: bool,
    pub content_complete: bool,
    pub trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceReadinessObservation {
    pub required: bool,
    pub ready: bool,
    pub source_identity_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubstituterObservation {
    pub configured: bool,
    pub candidate_available: bool,
    pub trusted: bool,
    pub requires_network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArchiveObservation {
    pub configured: bool,
    pub candidate_available: bool,
    pub prefix_matches: bool,
    pub signature_trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RemoteCandidateObservation {
    pub candidate_identity_blake3: String,
    pub configured: bool,
    pub credential_present: bool,
    pub capabilities_match: bool,
    pub source_inputs_ready: bool,
    pub output_trusted: bool,
    pub requires_network: bool,
    pub upload_summary: crate::UploadSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DoctorObservation {
    pub preflight_ok: bool,
    pub report_identity_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlatformObservation {
    pub supported: bool,
    pub platform_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TrustObservation {
    pub trusted_output_key_count: u32,
    pub trust_policy_identity_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NetworkObservation {
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutorObservation {
    pub local_available: bool,
    pub remote_available: bool,
    pub executor_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanningFacts {
    pub local_output: ObservedFact<LocalOutputObservation>,
    pub source_readiness: ObservedFact<SourceReadinessObservation>,
    pub substituter: ObservedFact<SubstituterObservation>,
    pub archive: ObservedFact<ArchiveObservation>,
    pub remote_candidate: ObservedFact<RemoteCandidateObservation>,
    pub doctor: ObservedFact<DoctorObservation>,
    pub platform: ObservedFact<PlatformObservation>,
    pub trust: ObservedFact<TrustObservation>,
    pub network: ObservedFact<NetworkObservation>,
    pub executor: ObservedFact<ExecutorObservation>,
}
