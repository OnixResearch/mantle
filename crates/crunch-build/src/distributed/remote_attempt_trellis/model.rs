use serde::Deserialize;
use serde::Serialize;

pub const TRELLIS_FENCED_ATTEMPT_REVISION: &str = "8de4b24aa2d66cc2e6ec966d686df023492265d3";
pub const TRELLIS_FENCED_ATTEMPT_SOURCE_ARCHIVE_BLAKE3: &str =
    "e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c";
pub const TRELLIS_REMOTE_ADMISSION_ORACLE_SCHEMA: &str = "mantle-trellis-remote-admission-oracle-v1";
pub const TRELLIS_REMOTE_ADMISSION_MATRIX_CASES: u32 = 6_720;
pub const TRELLIS_REMOTE_ADMISSION_ORACLE_RECORD_BYTES: usize = 7;
pub(super) const TRELLIS_EVENT_LIMIT: usize = 4_096;

const _: () = assert!(TRELLIS_REMOTE_ADMISSION_MATRIX_CASES > 0);
const _: () = assert!(TRELLIS_REMOTE_ADMISSION_ORACLE_RECORD_BYTES > 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisProjectionPhase {
    Queued,
    Running,
    Transferring,
    ResultReady,
    Completed,
    Failed,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisProjectionReportKind {
    Start,
    Heartbeat,
    LogAppend,
    Checkpoint,
    ResultReady,
    Failure,
    Completion,
}

impl TrellisProjectionReportKind {
    #[must_use]
    pub const fn trellis_kind(self) -> TrellisModelReportKind {
        match self {
            Self::Start => TrellisModelReportKind::Start,
            Self::Heartbeat | Self::LogAppend => TrellisModelReportKind::Progress,
            Self::Checkpoint => TrellisModelReportKind::Checkpoint,
            Self::ResultReady => TrellisModelReportKind::ResultReady,
            Self::Failure => TrellisModelReportKind::Failure,
            Self::Completion => TrellisModelReportKind::Completion,
        }
    }

    #[must_use]
    pub(super) const fn requires_worker_in_trellis(self) -> bool {
        !matches!(self, Self::Failure)
    }

    #[must_use]
    pub(super) const fn requires_output_in_mantle(self) -> bool {
        matches!(self, Self::ResultReady | Self::Completion)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisModelReportKind {
    Start,
    Progress,
    Checkpoint,
    ResultReady,
    Failure,
    Completion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisIdentityClass {
    Current,
    Stale,
    Future,
    WrongJob,
    WrongRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisEventClass {
    New,
    Replayed,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisHistoryClass {
    Available,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisScopeClass {
    Granted,
    Denied,
}

impl TrellisScopeClass {
    #[must_use]
    pub(super) const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisResultLinkageClass {
    NotApplicable,
    MissingRetained,
    Matching,
    Mismatched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisProjectionError {
    StateNotAdmitted,
    ReportNotAdmitted,
    PhaseReportMismatch,
    AuthorityOrderMismatch,
    AuthorityMeaningMismatch,
    HistoryOrderMismatch,
    ResultRetentionMismatch,
    ReasonClassUnmapped,
    OutcomeMutation,
}

impl TrellisProjectionError {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StateNotAdmitted => "state-not-admitted",
            Self::ReportNotAdmitted => "report-not-admitted",
            Self::PhaseReportMismatch => "phase-report-mismatch",
            Self::AuthorityOrderMismatch => "authority-order-mismatch",
            Self::AuthorityMeaningMismatch => "authority-meaning-mismatch",
            Self::HistoryOrderMismatch => "history-order-mismatch",
            Self::ResultRetentionMismatch => "result-retention-mismatch",
            Self::ReasonClassUnmapped => "reason-class-unmapped",
            Self::OutcomeMutation => "outcome-mutation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrellisRemoteAttemptCase {
    pub phase: TrellisProjectionPhase,
    pub report_kind: TrellisProjectionReportKind,
    pub identity: TrellisIdentityClass,
    pub event: TrellisEventClass,
    pub history: TrellisHistoryClass,
    pub worker: TrellisScopeClass,
    pub output: TrellisScopeClass,
    pub result_linkage: TrellisResultLinkageClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisRemoteAttemptProjection {
    pub case: TrellisRemoteAttemptCase,
    pub progress_before: u32,
    pub event_count_before: u32,
    pub result_present_before: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisOutcomeDisposition {
    Applied,
    Replayed,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisRejectClass {
    StaleEpoch,
    FutureEpoch,
    WrongJob,
    WrongRun,
    EventConflict,
    TerminalPhase,
    InvalidTransition,
    WorkerDenied,
    OutputDenied,
    MissingResult,
    ResultMismatch,
    EpochExhausted,
    HistoryFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrellisResultEffect {
    Preserved,
    SetFromReport,
    Cleared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisNormalizedOutcome {
    pub disposition: TrellisOutcomeDisposition,
    pub reject_class: Option<TrellisRejectClass>,
    pub next_phase: TrellisProjectionPhase,
    pub progress_delta: u8,
    pub event_delta: u8,
    pub result_effect: TrellisResultEffect,
    pub state_preserved: bool,
}
