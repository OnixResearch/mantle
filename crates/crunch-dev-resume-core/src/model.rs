use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

pub const DEV_RESUME_BUNDLE_SCHEMA: &str = "mantle-dev-stage-resume-bundle-v1";
pub const DEV_RESUME_REPORT_SCHEMA: &str = "mantle-dev-stage-resume-report-v1";
pub const DEV_RESUME_STAGE_COUNT: u32 = 6;
pub(crate) const DEV_RESUME_STAGE_COUNT_USIZE: usize = 6;
pub const DEV_RESUME_CANDIDATES_MAX: u32 = 16;
pub(crate) const DEV_RESUME_CANDIDATES_MAX_USIZE: usize = 16;
pub const DEV_RESUME_PAYLOADS_MAX: u32 = 32;
pub const DEV_RESUME_TEXT_BYTES_MAX: u32 = 4_096;
pub const BLAKE3_HEX_LENGTH_CHARS: u32 = 64;
pub const PROVIDER_CHECKPOINT_PAYLOAD_ID: &str = "provider-checkpoint";
pub const MANTLE_STAGE1_PAYLOAD_ID: &str = "mantle-stage1";
pub const FIXED_POINT_COMPLETE_PAYLOAD_ID: &str = "fixed-point-complete";
const STAGEX_TRANSITION_INDEX: usize = 0;
const STAGEX_PROVIDER_INDEX: usize = 1;
const FULL_SOURCE_NATIVE_INDEX: usize = 2;
const FULL_SOURCE_RUST_INDEX: usize = 3;
const MANTLE_STAGE1_INDEX: usize = 4;
const MANTLE_STAGE2_INDEX: usize = 5;
const STAGEX_TRANSITION_ORDINAL: u32 = 0;
const STAGEX_PROVIDER_ORDINAL: u32 = 1;
const FULL_SOURCE_NATIVE_ORDINAL: u32 = 2;
const FULL_SOURCE_RUST_ORDINAL: u32 = 3;
const MANTLE_STAGE1_ORDINAL: u32 = 4;
const MANTLE_STAGE2_ORDINAL: u32 = 5;
const STAGEX_TRANSITION_CODE: u8 = 1;
const STAGEX_PROVIDER_CODE: u8 = 2;
const FULL_SOURCE_NATIVE_CODE: u8 = 3;
const FULL_SOURCE_RUST_CODE: u8 = 4;
const MANTLE_STAGE1_CODE: u8 = 5;
const MANTLE_STAGE2_CODE: u8 = 6;
const PRESERVED_TREE_CODE: u8 = 1;
const DIRECTORY_CODE: u8 = 2;
const REGULAR_FILE_CODE: u8 = 3;
const CONTENT_REFERENCE_CODE: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResumeStage {
    StagexTransition,
    StagexProvider,
    FullSourceNativeProvider,
    FullSourceRustProvider,
    MantleStage1,
    MantleStage2,
}

impl ResumeStage {
    pub const ALL: [Self; DEV_RESUME_STAGE_COUNT_USIZE] = [
        Self::StagexTransition,
        Self::StagexProvider,
        Self::FullSourceNativeProvider,
        Self::FullSourceRustProvider,
        Self::MantleStage1,
        Self::MantleStage2,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::StagexTransition => "stagex-transition",
            Self::StagexProvider => "stagex-provider-publication",
            Self::FullSourceNativeProvider => "full-source-native-provider",
            Self::FullSourceRustProvider => "full-source-rust-provider",
            Self::MantleStage1 => "mantle-stage1",
            Self::MantleStage2 => "mantle-stage2",
        }
    }

    pub const fn ordinal(self) -> u32 {
        match self {
            Self::StagexTransition => STAGEX_TRANSITION_ORDINAL,
            Self::StagexProvider => STAGEX_PROVIDER_ORDINAL,
            Self::FullSourceNativeProvider => FULL_SOURCE_NATIVE_ORDINAL,
            Self::FullSourceRustProvider => FULL_SOURCE_RUST_ORDINAL,
            Self::MantleStage1 => MANTLE_STAGE1_ORDINAL,
            Self::MantleStage2 => MANTLE_STAGE2_ORDINAL,
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::StagexTransition => STAGEX_TRANSITION_INDEX,
            Self::StagexProvider => STAGEX_PROVIDER_INDEX,
            Self::FullSourceNativeProvider => FULL_SOURCE_NATIVE_INDEX,
            Self::FullSourceRustProvider => FULL_SOURCE_RUST_INDEX,
            Self::MantleStage1 => MANTLE_STAGE1_INDEX,
            Self::MantleStage2 => MANTLE_STAGE2_INDEX,
        }
    }

    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::StagexTransition => STAGEX_TRANSITION_CODE,
            Self::StagexProvider => STAGEX_PROVIDER_CODE,
            Self::FullSourceNativeProvider => FULL_SOURCE_NATIVE_CODE,
            Self::FullSourceRustProvider => FULL_SOURCE_RUST_CODE,
            Self::MantleStage1 => MANTLE_STAGE1_CODE,
            Self::MantleStage2 => MANTLE_STAGE2_CODE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResumeRunMode {
    Dev,
    Promoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResumePayloadKind {
    PreservedTree,
    Directory,
    RegularFile,
    ContentReference,
}

impl ResumePayloadKind {
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::PreservedTree => PRESERVED_TREE_CODE,
            Self::Directory => DIRECTORY_CODE,
            Self::RegularFile => REGULAR_FILE_CODE,
            Self::ContentReference => CONTENT_REFERENCE_CODE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePolicyDigests {
    pub closure_policy_digest_blake3: String,
    pub hermeticity_policy_digest_blake3: String,
    pub protected_execution_policy_digest_blake3: String,
    pub effect_policy_digest_blake3: String,
    pub normalization_policy_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeStageBinding {
    pub stage: ResumeStage,
    pub stage_id: String,
    pub producer_executable_digest_blake3: String,
    pub output_digest_blake3: String,
    pub execution_evidence_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumePayloadBinding {
    pub payload_id: String,
    pub destination_relative_path: String,
    pub kind: ResumePayloadKind,
    pub digest_blake3: String,
    pub total_file_bytes: u64,
    pub entry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeBundleManifest {
    pub schema: String,
    pub bundle_identity_blake3: String,
    pub source_authority_digest_blake3: String,
    pub plan_digest_blake3: String,
    pub policy_digest_blake3: String,
    pub completed_stage: ResumeStageBinding,
    pub payloads: Vec<ResumePayloadBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeCandidateObservation {
    pub manifest: ResumeBundleManifest,
    pub current_source_authority_digest_blake3: String,
    pub current_plan_digest_blake3: String,
    pub current_policy_digest_blake3: String,
    pub current_stage: ResumeStageBinding,
    pub observed_payloads: Vec<ResumePayloadBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePlanInput {
    pub mode: ResumeRunMode,
    pub candidates: Vec<ResumeCandidateObservation>,
    pub observed_rejections: Vec<RejectedResumeCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResumeDisposition {
    Restore,
    ExecuteCold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResumeRejectReason {
    NoCandidate,
    PromotedMode,
    CandidateLimitExceeded,
    SchemaMismatch,
    ManifestInvalid,
    BundleIdentityMismatch,
    SourceMismatch,
    PlanMismatch,
    PolicyMismatch,
    StageMismatch,
    ProducerMismatch,
    OutputMismatch,
    ExecutionEvidenceMismatch,
    PayloadSetIncomplete,
    PayloadMismatch,
    ConflictingCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedResumeCandidate {
    pub bundle_identity_blake3: String,
    pub completed_stage: ResumeStage,
    pub reason: ResumeRejectReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumePlan {
    pub disposition: ResumeDisposition,
    pub selected_bundle_identity_blake3: Option<String>,
    pub completed_stage: Option<ResumeStage>,
    pub restored_stages: Vec<ResumeStage>,
    pub executed_stages: Vec<ResumeStage>,
    pub first_incomplete_stage: Option<ResumeStage>,
    pub cold_reason: Option<ResumeRejectReason>,
    pub rejected_candidates: Vec<RejectedResumeCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevResumeReport {
    pub schema: String,
    pub status: String,
    pub mode: ResumeRunMode,
    pub plan_digest_blake3: String,
    pub selected_bundle_identity_blake3: Option<String>,
    pub published_bundle_identities_blake3: Vec<String>,
    pub restored_stages: Vec<ResumeStage>,
    pub executed_stages: Vec<ResumeStage>,
    pub first_incomplete_stage: Option<ResumeStage>,
    pub rejected_candidates: Vec<RejectedResumeCandidate>,
    pub cache_adoption_disposition: String,
    pub promoted_receipt_written: bool,
    pub release_alias_updated: bool,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeError {
    message: String,
}

impl ResumeError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ResumeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
