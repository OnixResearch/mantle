use alloc::vec::Vec;

use crate::types::BoundedDiagnostic;
use crate::types::DispatchDecision;
use crate::types::FailureFact;
use crate::types::FailureScope;
use crate::types::OutcomeError;
use crate::types::RootOutcome;
use crate::types::RootReferences;
use crate::types::RootSet;
use crate::types::RunSummary;
use crate::types::SelectedRoot;
use crate::types::SourceSequence;
use crate::types::TerminalPhase;
use crate::types::TerminalState;
use crate::types::TransitionResult;
use crate::types::stopped_outcome;
use crate::types::success_outcome;
use crate::types::summary;

const CANONICAL_ORDER_WINDOW: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RootProgress {
    Pending,
    Started,
    Terminal(RootOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RootSlot {
    root: SelectedRoot,
    progress: RootProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeLedger {
    root_set: RootSet,
    slots: Vec<RootSlot>,
}

impl OutcomeLedger {
    pub fn new(root_set: RootSet) -> Self {
        let slots: Vec<RootSlot> = root_set
            .roots
            .iter()
            .cloned()
            .map(|root| RootSlot {
                root,
                progress: RootProgress::Pending,
            })
            .collect();
        debug_assert_eq!(root_set.roots.len(), slots.len());
        debug_assert!(!slots.is_empty());
        Self { root_set, slots }
    }

    pub fn root_set(&self) -> RootSet {
        self.root_set.clone()
    }

    pub fn start(&self, sequence: SourceSequence) -> Result<Self, OutcomeError> {
        let index = self.slot_index(sequence)?;
        let mut next = self.clone();
        match &next.slots[index].progress {
            RootProgress::Pending => next.slots[index].progress = RootProgress::Started,
            RootProgress::Started => return Err(OutcomeError::RootAlreadyStarted(sequence.value())),
            RootProgress::Terminal(_) => return Err(OutcomeError::DuplicateTerminal(sequence.value())),
        }
        debug_assert!(matches!(next.slots[index].progress, RootProgress::Started));
        debug_assert_eq!(next.slots.len(), self.slots.len());
        Ok(next)
    }

    pub fn record_success(
        &self,
        sequence: SourceSequence,
        references: RootReferences,
    ) -> Result<TransitionResult, OutcomeError> {
        self.record_success_at_phase(sequence, TerminalPhase::Evaluation, references)
    }

    pub fn record_success_at_phase(
        &self,
        sequence: SourceSequence,
        terminal_phase: TerminalPhase,
        references: RootReferences,
    ) -> Result<TransitionResult, OutcomeError> {
        let root = self.root(sequence)?;
        let outcome = success_outcome(root, terminal_phase, references);
        let ledger = self.record_terminal(sequence, outcome)?;
        Ok(TransitionResult {
            ledger,
            decision: DispatchDecision::Continue,
        })
    }

    pub fn record_failure(
        &self,
        sequence: SourceSequence,
        fact: FailureFact,
        diagnostic: BoundedDiagnostic,
    ) -> Result<TransitionResult, OutcomeError> {
        let scope = fact.scope();
        let terminal_phase = fact.phase();
        let terminal_state = state_for_observed_failure(scope);
        let root = self.root(sequence)?;
        let outcome = stopped_outcome(root, terminal_state, terminal_phase, scope, diagnostic.clone());
        let ledger = self.record_terminal(sequence, outcome)?;
        if scope == FailureScope::RootScoped {
            return Ok(TransitionResult {
                ledger,
                decision: DispatchDecision::Continue,
            });
        }
        let stopped = ledger.classify_remaining(scope, terminal_phase, diagnostic)?;
        Ok(TransitionResult {
            ledger: stopped,
            decision: DispatchDecision::Stop,
        })
    }

    pub fn stop_remaining(
        &self,
        scope: FailureScope,
        diagnostic: BoundedDiagnostic,
    ) -> Result<TransitionResult, OutcomeError> {
        self.stop_remaining_at_phase(scope, default_stop_phase(scope), diagnostic)
    }

    pub fn stop_remaining_at_phase(
        &self,
        scope: FailureScope,
        terminal_phase: TerminalPhase,
        diagnostic: BoundedDiagnostic,
    ) -> Result<TransitionResult, OutcomeError> {
        if scope == FailureScope::RootScoped {
            return Err(OutcomeError::StopScopeRequired);
        }
        let ledger = self.classify_remaining(scope, terminal_phase, diagnostic)?;
        Ok(TransitionResult {
            ledger,
            decision: DispatchDecision::Stop,
        })
    }

    pub fn outcomes(&self) -> Vec<RootOutcome> {
        self.slots
            .iter()
            .filter_map(|slot| match &slot.progress {
                RootProgress::Terminal(outcome) => Some(outcome.clone()),
                RootProgress::Pending | RootProgress::Started => None,
            })
            .collect()
    }

    pub fn finish(&self) -> Result<RunSummary, OutcomeError> {
        let outcomes = self.outcomes();
        if outcomes.len() != self.slots.len() {
            return Err(OutcomeError::IncompleteSummary);
        }
        debug_assert_eq!(outcomes.len(), self.root_set.roots.len());
        debug_assert!(outcomes.windows(CANONICAL_ORDER_WINDOW).all(outcomes_are_ordered));
        summary(&self.root_set, outcomes)
    }

    fn classify_remaining(
        &self,
        scope: FailureScope,
        terminal_phase: TerminalPhase,
        diagnostic: BoundedDiagnostic,
    ) -> Result<Self, OutcomeError> {
        let mut next = self.clone();
        for slot in &mut next.slots {
            let terminal_state = match slot.progress {
                RootProgress::Pending => state_for_pending_stop(scope)?,
                RootProgress::Started => state_for_started_stop(scope)?,
                RootProgress::Terminal(_) => continue,
            };
            slot.progress = RootProgress::Terminal(stopped_outcome(
                slot.root.clone(),
                terminal_state,
                terminal_phase,
                scope,
                diagnostic.clone(),
            ));
        }
        debug_assert_eq!(next.slots.len(), self.slots.len());
        debug_assert!(next.slots.iter().all(slot_is_terminal));
        Ok(next)
    }

    fn record_terminal(&self, sequence: SourceSequence, outcome: RootOutcome) -> Result<Self, OutcomeError> {
        let index = self.slot_index(sequence)?;
        let mut next = self.clone();
        match &next.slots[index].progress {
            RootProgress::Pending => return Err(OutcomeError::RootNotStarted(sequence.value())),
            RootProgress::Started => next.slots[index].progress = RootProgress::Terminal(outcome),
            RootProgress::Terminal(_) => return Err(OutcomeError::DuplicateTerminal(sequence.value())),
        }
        debug_assert!(matches!(next.slots[index].progress, RootProgress::Terminal(_)));
        debug_assert_eq!(next.slots.len(), self.slots.len());
        Ok(next)
    }

    fn root(&self, sequence: SourceSequence) -> Result<SelectedRoot, OutcomeError> {
        let index = self.slot_index(sequence)?;
        Ok(self.slots[index].root.clone())
    }

    fn slot_index(&self, sequence: SourceSequence) -> Result<usize, OutcomeError> {
        let index = usize::try_from(sequence.value()).map_err(|_| OutcomeError::ArithmeticOverflow)?;
        if index >= self.slots.len() {
            return Err(OutcomeError::SequenceOutOfRange(sequence.value()));
        }
        Ok(index)
    }
}

fn default_stop_phase(scope: FailureScope) -> TerminalPhase {
    match scope {
        FailureScope::SharedFatal => TerminalPhase::Evaluation,
        FailureScope::Cancellation | FailureScope::CoordinatorFailure | FailureScope::RootScoped => {
            TerminalPhase::Coordination
        }
    }
}

fn state_for_observed_failure(scope: FailureScope) -> TerminalState {
    match scope {
        FailureScope::RootScoped | FailureScope::SharedFatal => TerminalState::Failed,
        FailureScope::Cancellation => TerminalState::Cancelled,
        FailureScope::CoordinatorFailure => TerminalState::WorkerLost,
    }
}

fn state_for_pending_stop(scope: FailureScope) -> Result<TerminalState, OutcomeError> {
    match scope {
        FailureScope::SharedFatal | FailureScope::Cancellation | FailureScope::CoordinatorFailure => {
            Ok(TerminalState::NotStarted)
        }
        FailureScope::RootScoped => Err(OutcomeError::StopScopeRequired),
    }
}

fn state_for_started_stop(scope: FailureScope) -> Result<TerminalState, OutcomeError> {
    match scope {
        FailureScope::SharedFatal => Ok(TerminalState::Failed),
        FailureScope::Cancellation => Ok(TerminalState::Cancelled),
        FailureScope::CoordinatorFailure => Ok(TerminalState::WorkerLost),
        FailureScope::RootScoped => Err(OutcomeError::StopScopeRequired),
    }
}

fn slot_is_terminal(slot: &RootSlot) -> bool {
    matches!(slot.progress, RootProgress::Terminal(_))
}

fn outcomes_are_ordered(pair: &[RootOutcome]) -> bool {
    let [left, right] = pair else {
        return false;
    };
    left.root.sequence < right.root.sequence
}
