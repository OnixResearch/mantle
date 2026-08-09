use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use crate::CONTEXT_FIELD_BYTES_MAX_USIZE;
use crate::DIAGNOSTIC_BYTES_MAX_USIZE;
use crate::REFERENCE_BYTES_MAX_USIZE;
use crate::ROOT_LABEL_BYTES_MAX_USIZE;
use crate::ROOTS_MAX;
use crate::ROOTS_MAX_USIZE;
use crate::STREAM_SCHEMA;
use crate::identity::root_identity;
use crate::identity::run_identity;

const BLAKE3_HEX_BYTES: usize = 64;

#[derive(Debug, Clone, Copy)]
struct TextBound {
    field: &'static str,
    bytes_max: usize,
    required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutcomeError {
    EmptyRootSet,
    TooManyRoots,
    DuplicateRootLabel(String),
    EmptyField(&'static str),
    FieldTooLong(&'static str),
    InvalidSourceIdentity,
    SequenceOutOfRange(u32),
    RootAlreadyStarted(u32),
    RootNotStarted(u32),
    DuplicateTerminal(u32),
    IncompleteSummary,
    MissingSuccessReference,
    EmptyReference,
    StopScopeRequired,
    ArithmeticOverflow,
}

impl fmt::Display for OutcomeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRootSet => write!(formatter, "selected root set is empty"),
            Self::TooManyRoots => write!(formatter, "selected root count exceeds {ROOTS_MAX}"),
            Self::DuplicateRootLabel(label) => write!(formatter, "duplicate root label: {label}"),
            Self::EmptyField(field) => write!(formatter, "{field} is empty"),
            Self::FieldTooLong(field) => write!(formatter, "{field} exceeds its byte limit"),
            Self::InvalidSourceIdentity => write!(formatter, "source identity is not lowercase BLAKE3 hex"),
            Self::SequenceOutOfRange(sequence) => write!(formatter, "sequence is out of range: {sequence}"),
            Self::RootAlreadyStarted(sequence) => write!(formatter, "root is already started: {sequence}"),
            Self::RootNotStarted(sequence) => write!(formatter, "root is not started: {sequence}"),
            Self::DuplicateTerminal(sequence) => write!(formatter, "root is already terminal: {sequence}"),
            Self::IncompleteSummary => write!(formatter, "not every selected root is terminal"),
            Self::MissingSuccessReference => write!(formatter, "successful root has no result reference"),
            Self::EmptyReference => write!(formatter, "root reference is empty"),
            Self::StopScopeRequired => write!(formatter, "remaining-root classification needs a stop scope"),
            Self::ArithmeticOverflow => write!(formatter, "checked outcome arithmetic overflowed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceSequence(u32);

impl SourceSequence {
    pub fn new(value: u32) -> Result<Self, OutcomeError> {
        if value >= ROOTS_MAX {
            return Err(OutcomeError::SequenceOutOfRange(value));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityContext {
    pub(crate) evaluator_cohort: String,
    pub(crate) source_blake3: String,
    pub(crate) selector: String,
}

impl IdentityContext {
    pub fn new(evaluator_cohort: String, source_blake3: String, selector: String) -> Result<Self, OutcomeError> {
        validate_text(&evaluator_cohort, TextBound {
            field: "evaluator cohort",
            bytes_max: CONTEXT_FIELD_BYTES_MAX_USIZE,
            required: true,
        })?;
        validate_text(&selector, TextBound {
            field: "selector",
            bytes_max: CONTEXT_FIELD_BYTES_MAX_USIZE,
            required: false,
        })?;
        validate_source_identity(&source_blake3)?;
        Ok(Self {
            evaluator_cohort,
            source_blake3,
            selector,
        })
    }

    pub fn evaluator_cohort(&self) -> String {
        self.evaluator_cohort.clone()
    }

    pub fn source_blake3(&self) -> String {
        self.source_blake3.clone()
    }

    pub fn selector(&self) -> String {
        self.selector.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedRoot {
    pub(crate) label: String,
    pub(crate) sequence: SourceSequence,
    pub(crate) root_id: String,
}

impl SelectedRoot {
    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn sequence(&self) -> SourceSequence {
        self.sequence
    }

    pub fn root_id(&self) -> String {
        self.root_id.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSet {
    pub(crate) context: IdentityContext,
    pub(crate) roots: Vec<SelectedRoot>,
    root_count: u32,
}

impl RootSet {
    pub fn admit(context: IdentityContext, labels: Vec<String>) -> Result<Self, OutcomeError> {
        let root_count = validate_root_count(labels.len())?;
        let mut roots = Vec::with_capacity(labels.len());
        for (index, label) in labels.into_iter().enumerate() {
            validate_text(&label, TextBound {
                field: "root label",
                bytes_max: ROOT_LABEL_BYTES_MAX_USIZE,
                required: true,
            })?;
            reject_duplicate_label(&roots, &label)?;
            let sequence_value = u32::try_from(index).map_err(|_| OutcomeError::TooManyRoots)?;
            let sequence = SourceSequence::new(sequence_value)?;
            let root_id = root_identity(&context, &label, sequence)?;
            roots.push(SelectedRoot {
                label,
                sequence,
                root_id,
            });
        }
        debug_assert!(!roots.is_empty());
        debug_assert!(roots.len() <= ROOTS_MAX_USIZE);
        Ok(Self {
            context,
            roots,
            root_count,
        })
    }

    pub fn context(&self) -> IdentityContext {
        self.context.clone()
    }

    pub fn roots(&self) -> Vec<SelectedRoot> {
        self.roots.clone()
    }

    pub fn root_count(&self) -> u32 {
        self.root_count
    }

    pub fn run_id(&self) -> Result<String, OutcomeError> {
        run_identity(&self.context, &self.roots)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureScope {
    RootScoped,
    SharedFatal,
    Cancellation,
    CoordinatorFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureFact {
    RootEvaluation,
    RootConversion,
    SharedSource,
    SharedImport,
    SharedEvaluatorCohort,
    SharedProtocol,
    Coordinator,
    OperatorCancellation,
}

impl FailureFact {
    pub fn scope(self) -> FailureScope {
        match self {
            Self::RootEvaluation | Self::RootConversion => FailureScope::RootScoped,
            Self::SharedSource | Self::SharedImport | Self::SharedEvaluatorCohort | Self::SharedProtocol => {
                FailureScope::SharedFatal
            }
            Self::Coordinator => FailureScope::CoordinatorFailure,
            Self::OperatorCancellation => FailureScope::Cancellation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalState {
    Succeeded,
    Failed,
    WorkerLost,
    Cancelled,
    NotStarted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedDiagnostic {
    pub(crate) text: String,
    pub(crate) truncated: bool,
}

impl BoundedDiagnostic {
    pub fn new(text: String) -> Self {
        let bytes_max = DIAGNOSTIC_BYTES_MAX_USIZE;
        if text.len() <= bytes_max {
            return Self { text, truncated: false };
        }
        let boundary = utf8_boundary(&text, bytes_max);
        Self {
            text: text[..boundary].to_string(),
            truncated: true,
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootReferences {
    pub(crate) result_ref: Option<String>,
    pub(crate) cache_ref: Option<String>,
    pub(crate) build_ref: Option<String>,
}

impl RootReferences {
    pub fn new(
        result_ref: Option<String>,
        cache_ref: Option<String>,
        build_ref: Option<String>,
    ) -> Result<Self, OutcomeError> {
        validate_reference(&result_ref)?;
        validate_reference(&cache_ref)?;
        validate_reference(&build_ref)?;
        if result_ref.is_none() && cache_ref.is_none() && build_ref.is_none() {
            return Err(OutcomeError::MissingSuccessReference);
        }
        Ok(Self {
            result_ref,
            cache_ref,
            build_ref,
        })
    }

    pub fn result_ref(&self) -> Option<String> {
        self.result_ref.clone()
    }

    pub fn cache_ref(&self) -> Option<String> {
        self.cache_ref.clone()
    }

    pub fn build_ref(&self) -> Option<String> {
        self.build_ref.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootOutcome {
    pub(crate) root: SelectedRoot,
    pub(crate) terminal_state: TerminalState,
    pub(crate) failure_scope: Option<FailureScope>,
    pub(crate) diagnostic: Option<BoundedDiagnostic>,
    pub(crate) references: Option<RootReferences>,
}

impl RootOutcome {
    pub fn root(&self) -> SelectedRoot {
        self.root.clone()
    }

    pub fn terminal_state(&self) -> TerminalState {
        self.terminal_state
    }

    pub fn failure_scope(&self) -> Option<FailureScope> {
        self.failure_scope
    }

    pub fn diagnostic(&self) -> Option<BoundedDiagnostic> {
        self.diagnostic.clone()
    }

    pub fn references(&self) -> Option<RootReferences> {
        self.references.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDisposition {
    Success,
    Partial,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RootCounts {
    pub succeeded: u32,
    pub failed: u32,
    pub worker_lost: u32,
    pub cancelled: u32,
    pub not_started: u32,
}

impl RootCounts {
    pub fn total(self) -> Result<u32, OutcomeError> {
        let terminal_failures = self.failed.checked_add(self.worker_lost).ok_or(OutcomeError::ArithmeticOverflow)?;
        let stopped = self.cancelled.checked_add(self.not_started).ok_or(OutcomeError::ArithmeticOverflow)?;
        self.succeeded
            .checked_add(terminal_failures)
            .and_then(|count| count.checked_add(stopped))
            .ok_or(OutcomeError::ArithmeticOverflow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub(crate) schema: String,
    pub(crate) run_id: String,
    pub(crate) disposition: RunDisposition,
    pub(crate) counts: RootCounts,
    pub(crate) roots: Vec<RootOutcome>,
}

impl RunSummary {
    pub fn schema(&self) -> String {
        self.schema.clone()
    }

    pub fn run_id(&self) -> String {
        self.run_id.clone()
    }

    pub fn disposition(&self) -> RunDisposition {
        self.disposition
    }

    pub fn counts(&self) -> RootCounts {
        self.counts
    }

    pub fn roots(&self) -> Vec<RootOutcome> {
        self.roots.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult {
    pub(crate) ledger: crate::OutcomeLedger,
    pub(crate) decision: DispatchDecision,
}

impl TransitionResult {
    pub fn into_ledger(self) -> crate::OutcomeLedger {
        self.ledger
    }

    pub fn decision(&self) -> DispatchDecision {
        self.decision
    }
}

pub(crate) fn terminal_counts(roots: &[RootOutcome]) -> Result<RootCounts, OutcomeError> {
    let mut counts = RootCounts::default();
    for root in roots {
        let target = match root.terminal_state {
            TerminalState::Succeeded => &mut counts.succeeded,
            TerminalState::Failed => &mut counts.failed,
            TerminalState::WorkerLost => &mut counts.worker_lost,
            TerminalState::Cancelled => &mut counts.cancelled,
            TerminalState::NotStarted => &mut counts.not_started,
        };
        *target = target.checked_add(1).ok_or(OutcomeError::ArithmeticOverflow)?;
    }
    Ok(counts)
}

pub(crate) fn disposition_for(counts: RootCounts, roots: &[RootOutcome]) -> Result<RunDisposition, OutcomeError> {
    if roots.iter().any(root_has_cancellation_scope) {
        return Ok(RunDisposition::Cancelled);
    }
    let total = counts.total()?;
    if counts.succeeded == total {
        return Ok(RunDisposition::Success);
    }
    if counts.succeeded > 0 {
        return Ok(RunDisposition::Partial);
    }
    Ok(RunDisposition::Failed)
}

fn root_has_cancellation_scope(root: &RootOutcome) -> bool {
    root.failure_scope == Some(FailureScope::Cancellation)
}

fn validate_root_count(count: usize) -> Result<u32, OutcomeError> {
    if count == 0 {
        return Err(OutcomeError::EmptyRootSet);
    }
    if count > ROOTS_MAX_USIZE {
        return Err(OutcomeError::TooManyRoots);
    }
    u32::try_from(count).map_err(|_| OutcomeError::TooManyRoots)
}

fn reject_duplicate_label(roots: &[SelectedRoot], label: &str) -> Result<(), OutcomeError> {
    if roots.iter().any(|root| root.label == label) {
        return Err(OutcomeError::DuplicateRootLabel(label.to_string()));
    }
    Ok(())
}

fn validate_text(text: &str, bound: TextBound) -> Result<(), OutcomeError> {
    if bound.required && text.is_empty() {
        return Err(OutcomeError::EmptyField(bound.field));
    }
    if text.len() > bound.bytes_max {
        return Err(OutcomeError::FieldTooLong(bound.field));
    }
    Ok(())
}

fn validate_source_identity(source_blake3: &str) -> Result<(), OutcomeError> {
    if source_blake3.len() != BLAKE3_HEX_BYTES {
        return Err(OutcomeError::InvalidSourceIdentity);
    }
    if !source_blake3.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(OutcomeError::InvalidSourceIdentity);
    }
    Ok(())
}

fn validate_reference(reference: &Option<String>) -> Result<(), OutcomeError> {
    let Some(reference) = reference else {
        return Ok(());
    };
    if reference.is_empty() {
        return Err(OutcomeError::EmptyReference);
    }
    if reference.len() > REFERENCE_BYTES_MAX_USIZE {
        return Err(OutcomeError::FieldTooLong("root reference"));
    }
    Ok(())
}

fn utf8_boundary(text: &str, bytes_max: usize) -> usize {
    let mut boundary = bytes_max.min(text.len());
    while !text.is_char_boundary(boundary) {
        boundary = boundary.saturating_sub(1);
    }
    boundary
}

pub(crate) fn success_outcome(root: SelectedRoot, references: RootReferences) -> RootOutcome {
    RootOutcome {
        root,
        terminal_state: TerminalState::Succeeded,
        failure_scope: None,
        diagnostic: None,
        references: Some(references),
    }
}

pub(crate) fn stopped_outcome(
    root: SelectedRoot,
    terminal_state: TerminalState,
    scope: FailureScope,
    diagnostic: BoundedDiagnostic,
) -> RootOutcome {
    debug_assert!(terminal_state != TerminalState::Succeeded);
    debug_assert!(scope != FailureScope::RootScoped || terminal_state == TerminalState::Failed);
    RootOutcome {
        root,
        terminal_state,
        failure_scope: Some(scope),
        diagnostic: Some(diagnostic),
        references: None,
    }
}

pub(crate) fn summary(root_set: &RootSet, roots: Vec<RootOutcome>) -> Result<RunSummary, OutcomeError> {
    let counts = terminal_counts(&roots)?;
    if counts.total()? != root_set.root_count() {
        return Err(OutcomeError::IncompleteSummary);
    }
    let disposition = disposition_for(counts, &roots)?;
    Ok(RunSummary {
        schema: STREAM_SCHEMA.to_string(),
        run_id: root_set.run_id()?,
        disposition,
        counts,
        roots,
    })
}
