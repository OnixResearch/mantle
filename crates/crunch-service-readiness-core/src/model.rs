use alloc::vec::Vec;

use crate::COORDINATION_CLASSIFICATION;
use crate::READINESS_SCHEMA;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentKind {
    Service,
    ProofStage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartPolicy {
    Always,
    OnError,
    All,
    Never,
}

impl RestartPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::OnError => "on-error",
            Self::All => "all",
            Self::Never => "never",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "always" => Some(Self::Always),
            "on-error" => Some(Self::OnError),
            "all" => Some(Self::All),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartAction {
    None,
    Component,
    Group,
}

impl RestartAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Component => "component",
            Self::Group => "group",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitKind {
    Normal,
    Abnormal,
}

/// Extra accepted authorities a proof stage must carry before `complete`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StageRequirements {
    pub require_action_reconciliation: bool,
    pub require_v2_receipt: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Component<'a> {
    pub id: &'a str,
    pub kind: ComponentKind,
    pub depends_on: &'a [&'a str],
    pub user_states: &'a [&'a str],
    pub restart_policy: Option<RestartPolicy>,
    pub stage_requirements: Option<StageRequirements>,
}

/// Shell-verified facts, not a signature or an authority granted by this core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProofCompletion<'a> {
    pub stage_evidence_digest_blake3: &'a str,
    pub output_digest_blake3: &'a str,
    pub execution_verified: bool,
    pub action_reconciled: bool,
    pub v2_receipt_verified: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation<'a> {
    pub id: &'a str,
    pub generation: u32,
    /// Local assertions. Graph-derived `ready` may be retracted in the report.
    pub states: &'a [&'a str],
    pub exit: Option<ExitKind>,
    /// Shell asserts only after a successful real request/subscription response.
    pub request_acknowledged: bool,
    pub proof_completion: Option<ProofCompletion<'a>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Snapshot<'a> {
    pub schema: &'a str,
    pub components: &'a [Component<'a>],
    pub observations: &'a [Observation<'a>],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessRow<'a> {
    pub id: &'a str,
    pub generation: Option<u32>,
    pub states: Vec<&'a str>,
    pub restart_policy: Option<RestartPolicy>,
    /// `None` means no exit was observed; `Some(None)` means an exit with no restart.
    pub restart_action: Option<RestartAction>,
    pub ready: bool,
    /// Only declared component IDs; source/policy blockers remain proof-shell facts.
    pub blocked_by: Vec<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessReport<'a> {
    pub schema: &'static str,
    pub classification: &'static str,
    pub evidence_eligible: bool,
    pub components: Vec<ReadinessRow<'a>>,
}

impl<'a> ReadinessReport<'a> {
    pub(crate) fn new(components: Vec<ReadinessRow<'a>>) -> Self {
        Self {
            schema: READINESS_SCHEMA,
            classification: COORDINATION_CLASSIFICATION,
            evidence_eligible: false,
            components,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    UnsupportedSchema,
    TooManyComponents,
    TooManyDependencies,
    InvalidId,
    DuplicateComponent,
    MissingPolicy,
    UnexpectedPolicy,
    MissingStageRequirements,
    UnexpectedStageRequirements,
    InvalidUserState,
    DuplicateUserState,
    UnknownDependency,
    DuplicateDependency,
    CyclicDependency,
    TooManyObservations,
    UnknownComponent,
    DuplicateObservation,
    InvalidGeneration,
    TooManyStates,
    UnknownState,
    DuplicateState,
    InvalidStateCombination,
    MissingRequestAcknowledgement,
    InvalidProofCompletion,
    InvalidProofDigest,
    InvalidPreviousReport,
    StaleGeneration,
    UnobservedStart,
    UnobservedExit,
    RevivedTerminal,
    RestartDenied,
    BlockedStart,
    BlockedCompletion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessError<'a> {
    pub code: ErrorCode,
    pub component: Option<&'a str>,
    pub related: Option<&'a str>,
}

impl<'a> ReadinessError<'a> {
    pub(crate) const fn new(code: ErrorCode, component: Option<&'a str>, related: Option<&'a str>) -> Self {
        Self {
            code,
            component,
            related,
        }
    }
}
