// machine-artifact-public: radiance.radiance-source-cohort
// machine-artifact-public: radiance.radiance-reference-receipt
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "kebab-case")]
pub enum RadianceSourceRole {
    Radiance,
    BootstrapCompiler,
    Emulator,
}

impl RadianceSourceRole {
    #[must_use]
    pub const fn expected_revision(self) -> &'static str {
        match self {
            Self::Radiance => crate::RADIANCE_REVISION,
            Self::BootstrapCompiler => crate::RADIANCE_S0_REVISION,
            Self::Emulator => crate::RADIANCE_EMULATOR_REVISION,
        }
    }

    #[must_use]
    pub const fn expected_content_blake3(self) -> &'static str {
        match self {
            Self::Radiance => crate::RADIANCE_CONTENT_BLAKE3,
            Self::BootstrapCompiler => crate::RADIANCE_S0_CONTENT_BLAKE3,
            Self::Emulator => crate::RADIANCE_EMULATOR_CONTENT_BLAKE3,
        }
    }

    #[must_use]
    pub const fn expected_repository_url(self) -> &'static str {
        match self {
            Self::Radiance => crate::RADIANCE_REPOSITORY_URL,
            Self::BootstrapCompiler => crate::RADIANCE_S0_REPOSITORY_URL,
            Self::Emulator => crate::RADIANCE_EMULATOR_REPOSITORY_URL,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceSourceMemberWire {
    pub role: RadianceSourceRole,
    pub repository_url: alloc::string::String,
    pub observation: crunch_source_core::SourceObservationWire,
    pub license_spdx: alloc::string::String,
    pub license_blake3: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceSourceCohortWire {
    pub schema: alloc::string::String,
    pub members: alloc::vec::Vec<RadianceSourceMemberWire>,
    pub cohort_blake3: alloc::string::String,
    pub non_claim: alloc::string::String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "kebab-case")]
pub enum RadianceRoute {
    Seed,
    C99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RadianceLaunchKind {
    NativeBootstrap,
    EmulatedCompiler,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceStagePlan {
    pub route: RadianceRoute,
    pub stage: u8,
    pub launch_kind: RadianceLaunchKind,
    pub predecessor_role: alloc::string::String,
    pub source_projection: alloc::string::String,
    pub output_role: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceReferencePlan {
    pub schema: alloc::string::String,
    pub source_cohort_blake3: alloc::string::String,
    pub source_bundle_blake3: alloc::string::String,
    pub stages: alloc::vec::Vec<RadianceStagePlan>,
    pub plan_blake3: alloc::string::String,
    pub non_claim: alloc::string::String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "kebab-case")]
pub enum RadianceArtifactRole {
    #[serde(rename = "host-c-compiler")]
    HostCompilerLauncher,
    #[serde(rename = "host-c-compiler-driver")]
    HostCompilerDriver,
    HostLinker,
    HostCrtInputs,
    HostLibgccInputs,
    BootstrapCompiler,
    Emulator,
    Seed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceArtifactObservation {
    pub role: RadianceArtifactRole,
    pub digest_blake3: alloc::string::String,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceStageObservation {
    pub route: RadianceRoute,
    pub stage: u8,
    pub launch_kind: RadianceLaunchKind,
    pub launcher_blake3: alloc::string::String,
    pub predecessor_blake3: alloc::string::String,
    pub output_role: alloc::string::String,
    pub output_blake3: alloc::string::String,
    pub output_bytes: u64,
    pub exit_code: i32,
    pub stdout_blake3: alloc::string::String,
    pub stderr_blake3: alloc::string::String,
    pub protected_audit_blake3: alloc::string::String,
    pub protected_audit_event_count: u32,
    pub denied_event_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceRouteConvergence {
    pub route: RadianceRoute,
    pub left_stage: u8,
    pub right_stage: u8,
    pub left_blake3: alloc::string::String,
    pub right_blake3: alloc::string::String,
    pub byte_count: u64,
    pub equal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RadianceCrossRouteDisposition {
    Match,
    Divergence,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceCrossRouteComparison {
    pub seed_blake3: alloc::string::String,
    pub c99_blake3: alloc::string::String,
    pub seed_bytes: u64,
    pub c99_bytes: u64,
    pub disposition: RadianceCrossRouteDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceZeroEventCounts {
    pub live_fetches: u32,
    pub source_fallbacks: u32,
    pub substitutions: u32,
    pub ambient_discoveries: u32,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "kebab-case")]
pub enum RadiancePublicationRole {
    SeedRouteFixedPoint,
    C99RouteFixedPoint,
    BootstrapCompiler,
    Emulator,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadiancePublicationArtifact {
    pub role: RadiancePublicationRole,
    pub relative_path: alloc::string::String,
    pub digest_blake3: alloc::string::String,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadianceReferenceReceiptDraft {
    pub source_cohort: RadianceSourceCohortWire,
    pub source_bundle_blake3: alloc::string::String,
    pub source_state_blake3: alloc::string::String,
    pub plan: RadianceReferencePlan,
    pub tools: alloc::vec::Vec<RadianceArtifactObservation>,
    pub stages: alloc::vec::Vec<RadianceStageObservation>,
    pub route_convergence: alloc::vec::Vec<RadianceRouteConvergence>,
    pub cross_route: RadianceCrossRouteComparison,
    pub zero_events: RadianceZeroEventCounts,
    pub publication: alloc::vec::Vec<RadiancePublicationArtifact>,
    pub protected_execution_audit_blake3: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadianceReferenceReceipt {
    pub schema: alloc::string::String,
    pub source_cohort: RadianceSourceCohortWire,
    pub source_bundle_blake3: alloc::string::String,
    pub source_state_blake3: alloc::string::String,
    pub plan: RadianceReferencePlan,
    pub tools: alloc::vec::Vec<RadianceArtifactObservation>,
    pub stages: alloc::vec::Vec<RadianceStageObservation>,
    pub route_convergence: alloc::vec::Vec<RadianceRouteConvergence>,
    pub cross_route: RadianceCrossRouteComparison,
    pub zero_events: RadianceZeroEventCounts,
    pub publication: alloc::vec::Vec<RadiancePublicationArtifact>,
    pub protected_execution_audit_blake3: alloc::string::String,
    pub receipt_blake3: alloc::string::String,
    pub non_claim: alloc::string::String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadianceReferenceError {
    Schema,
    SourceCount,
    SourceRole,
    SourceIdentity,
    SourceObservation,
    SourceLicense,
    Digest,
    Text,
    Plan,
    Tool,
    Stage,
    Lineage,
    Execution,
    Convergence,
    Publication,
    ZeroEvents,
    NonClaim,
    Canonicalization,
}
