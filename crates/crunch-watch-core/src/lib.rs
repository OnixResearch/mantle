//! Pure, bounded watch-session goal-set decisions. The shell owns evaluation,
//! scheduling, cancellation, clocks, file watching, and event publication.
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Matches the lazy scheduler's maximum number of tracked goals.
pub const MAX_LIVE_GOALS: u32 = 10_000;
/// A completely replaced admitted set emits at most two events per goal.
pub const MAX_TRANSITION_EVENTS: u32 = 20_000;
pub const MAX_GOAL_IDENTITY_BYTES: u32 = 4_096;
pub const MAX_RUN_IDENTITY_BYTES: u32 = 256;
pub const MAX_DIAGNOSTIC_BYTES: u32 = 16_384;
pub const MAX_EVALUATIONS_PER_WINDOW: u32 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchError {
    GoalLimit,
    EmptyGoalIdentity,
    GoalIdentityTooLong,
    EmptyRunIdentity,
    RunIdentityTooLong,
    EmptyDiagnostic,
    DiagnosticTooLong,
    EventBudget,
    EvaluationRate,
    StaleGeneration,
    GenerationOverflow,
}

/// The admitted scheduler identity, not a display label or source position.
/// Duplicate requests for the same identity are one goal, as in GoalRegistry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalSet {
    identities: Vec<String>,
}

impl GoalSet {
    pub fn admit(mut identities: Vec<String>) -> Result<Self, WatchError> {
        if identities.len() > MAX_LIVE_GOALS as usize {
            return Err(WatchError::GoalLimit);
        }
        for identity in &identities {
            if identity.is_empty() {
                return Err(WatchError::EmptyGoalIdentity);
            }
            if identity.len() > MAX_GOAL_IDENTITY_BYTES as usize {
                return Err(WatchError::GoalIdentityTooLong);
            }
        }
        identities.sort_unstable();
        identities.dedup();
        debug_assert!(identities.len() <= MAX_LIVE_GOALS as usize);
        debug_assert!(identities.windows(2).all(|pair| pair[0] < pair[1]));
        Ok(Self { identities })
    }

    pub fn identities(&self) -> &[String] {
        &self.identities
    }

    pub fn contains(&self, identity: &str) -> bool {
        self.identities.binary_search_by(|current| current.as_str().cmp(identity)).is_ok()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchEventKind {
    Added,
    Retained,
    Retracted,
    Rejected,
}

impl WatchEventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Retained => "retained",
            Self::Retracted => "retracted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WatchEvent {
    pub kind: WatchEventKind,
    /// Absent only for a failed evaluation/conversion/policy decision.
    pub goal_identity: Option<String>,
    /// Present only on a rejected generation. The shell must sanitize secrets.
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalDiff {
    /// One stable identity per watch process, not a new ID per generation.
    pub run_identity: String,
    /// Sorted by goal identity; each admitted identity appears exactly once.
    pub events: Vec<WatchEvent>,
}

fn checked_run_identity(run_identity: &str) -> Result<(), WatchError> {
    if run_identity.is_empty() {
        return Err(WatchError::EmptyRunIdentity);
    }
    if run_identity.len() > MAX_RUN_IDENTITY_BYTES as usize {
        return Err(WatchError::RunIdentityTooLong);
    }
    Ok(())
}

fn checked_event_budget(event_count: usize, event_budget: u32) -> Result<(), WatchError> {
    if event_budget > MAX_TRANSITION_EVENTS {
        return Err(WatchError::EventBudget);
    }
    if event_count > event_budget as usize {
        return Err(WatchError::EventBudget);
    }
    Ok(())
}

/// Classifies the two fully admitted sets. On any error, neither set changes.
/// The caller must apply the result only after evaluation, conversion, policy,
/// and goal admission all succeed; failed generations use `reject_generation`.
/// r[impl build_scheduling.watch_plan_assertion]
pub fn diff(
    previous: &GoalSet,
    current: &GoalSet,
    run_identity: &str,
    event_budget: u32,
) -> Result<GoalDiff, WatchError> {
    checked_run_identity(run_identity)?;
    // Bound allocation before processing. A retained identity emits only one
    // event; reject an exceeded budget while walking rather than overcounting.
    checked_event_budget(0, event_budget)?;
    let maximum_count = previous.identities.len() + current.identities.len();
    let mut events = Vec::with_capacity(maximum_count.min(event_budget as usize));
    let (mut old_index, mut new_index) = (0, 0);
    while old_index < previous.identities.len() || new_index < current.identities.len() {
        let old = previous.identities.get(old_index);
        let new = current.identities.get(new_index);
        let (kind, identity) = match (old, new) {
            (Some(old), Some(new)) if old == new => {
                old_index += 1;
                new_index += 1;
                (WatchEventKind::Retained, old)
            }
            (Some(old), Some(new)) if old < new => {
                old_index += 1;
                (WatchEventKind::Retracted, old)
            }
            (Some(_), Some(new)) | (None, Some(new)) => {
                new_index += 1;
                (WatchEventKind::Added, new)
            }
            (Some(old), None) => {
                old_index += 1;
                (WatchEventKind::Retracted, old)
            }
            (None, None) => break,
        };
        if events.len() >= event_budget as usize {
            return Err(WatchError::EventBudget);
        }
        events.push(WatchEvent {
            kind,
            goal_identity: Some(identity.clone()),
            diagnostic: None,
        });
    }
    debug_assert!(events.len() <= maximum_count);
    debug_assert!(events.len() <= event_budget as usize);
    Ok(GoalDiff {
        run_identity: run_identity.into(),
        events,
    })
}

/// A rejected generation never yields retractions, and the previous GoalSet
/// remains the shell's admitted set. A diagnostic is untrusted and MUST be
/// sanitized/redacted by the shell before publication.
/// r[impl build_scheduling.watch_error_retention]
pub fn reject_generation(run_identity: &str, diagnostic: String, event_budget: u32) -> Result<GoalDiff, WatchError> {
    checked_run_identity(run_identity)?;
    checked_event_budget(1, event_budget)?;
    if diagnostic.is_empty() {
        return Err(WatchError::EmptyDiagnostic);
    }
    if diagnostic.len() > MAX_DIAGNOSTIC_BYTES as usize {
        return Err(WatchError::DiagnosticTooLong);
    }
    Ok(GoalDiff {
        run_identity: run_identity.into(),
        events: alloc::vec![WatchEvent {
            kind: WatchEventKind::Rejected,
            goal_identity: None,
            diagnostic: Some(diagnostic),
        }],
    })
}

/// The shell counts attempted evaluations in its declared clock window;
/// this core never reads a clock. An attempt at the limit is rejected.
pub fn admit_evaluation(attempts_in_window: u32) -> Result<(), WatchError> {
    if attempts_in_window >= MAX_EVALUATIONS_PER_WINDOW {
        return Err(WatchError::EvaluationRate);
    }
    Ok(())
}

/// One watch process's last admitted goal set and newest uncommitted edit.
///
/// Successful edits coalesce before the shell publishes any retraction: a
/// later edit supersedes an earlier candidate, but neither changes the last
/// admitted set until `commit` succeeds. The shell may call `commit` only
/// after observing cancellation/teardown of every retracted building goal.
/// No clock, sandbox, or output authority exists in this pure state machine.
pub struct WatchLoop {
    admitted: GoalSet,
    pending: Option<PendingGeneration>,
    run_identity: String,
    last_generation: u64,
}

struct PendingGeneration {
    number: u64,
    current: GoalSet,
    transition: GoalDiff,
}

impl WatchLoop {
    pub fn new(run_identity: String, admitted: GoalSet) -> Result<Self, WatchError> {
        checked_run_identity(&run_identity)?;
        Ok(Self {
            admitted,
            pending: None,
            run_identity,
            last_generation: 0,
        })
    }

    pub fn admitted(&self) -> &GoalSet {
        &self.admitted
    }

    /// Stage a fully evaluated, converted, policy-admitted goal set. A failed
    /// stage invalidates a previously pending candidate, never admitted goals.
    pub fn offer(&mut self, current: GoalSet, event_budget: u32) -> Result<u64, WatchError> {
        self.pending = None;
        let transition = diff(&self.admitted, &current, &self.run_identity, event_budget)?;
        let number = self.last_generation.checked_add(1).ok_or(WatchError::GenerationOverflow)?;
        self.last_generation = number;
        self.pending = Some(PendingGeneration {
            number,
            current,
            transition,
        });
        Ok(number)
    }

    /// The latest candidate, not every intermediate file edit.
    pub fn pending(&self) -> Option<(u64, &GoalDiff)> {
        self.pending.as_ref().map(|candidate| (candidate.number, &candidate.transition))
    }

    /// Reject an invalid source generation while retaining all admitted goals.
    pub fn reject(&mut self, diagnostic: String, event_budget: u32) -> Result<GoalDiff, WatchError> {
        self.pending = None;
        reject_generation(&self.run_identity, diagnostic, event_budget)
    }

    /// Move the exact latest candidate into the admitted set. The shell MUST
    /// call this only after the worker proves every required sandbox teardown
    /// and blocks late output publication; a dropped future is not proof.
    pub fn commit(&mut self, number: u64) -> Result<(), WatchError> {
        if self.pending.as_ref().map(|candidate| candidate.number) != Some(number) {
            return Err(WatchError::StaleGeneration);
        }
        let pending = self.pending.take().expect("matching pending generation was checked");
        self.admitted = pending.current;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalExecutionState {
    Queued,
    Building,
    /// The shell observed sandbox child and descendants stopped; a timeout
    /// or a dropped future MUST NOT be represented as this state.
    Stopped,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetractionDecision {
    pub cancel_in_flight: bool,
    pub release_reservation: bool,
    pub admit_late_success: bool,
}

/// A retracted running goal cannot publish a late success. It retains its
/// reservation while cancellation is pending, including after the reporting
/// deadline; only observed teardown can authorize release.
/// r[impl build_scheduling.watch_retraction_cancellation]
pub const fn retract(state: GoalExecutionState) -> RetractionDecision {
    RetractionDecision {
        cancel_in_flight: matches!(state, GoalExecutionState::Building),
        release_reservation: !matches!(state, GoalExecutionState::Building),
        admit_late_success: false,
    }
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
