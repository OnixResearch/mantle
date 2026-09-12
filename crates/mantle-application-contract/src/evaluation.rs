//! Evaluation command family.
//!
//! Evaluation commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one evaluation command.
pub const MAX_EVALUATION_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one evaluation command beyond its declared entries.
const MAX_EVALUATION_BLOCKERS: usize = 4;

/// Evaluation operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvaluationOperation {
    /// Evaluate operation.
    Evaluate,
    /// Force operation.
    Force,
    /// Inspect operation.
    Inspect,
    /// Stream operation.
    Stream,
}

impl EvaluationOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Evaluate, Self::Force, Self::Inspect, Self::Stream];
        debug_assert_eq!(entries.len(), 4);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Evaluate => "evaluate",
            Self::Force => "force",
            Self::Inspect => "inspect",
            Self::Stream => "stream",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(EvaluationOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Force);
        debug_assert!(EvaluationOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed Evaluation command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: EvaluationOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Source path.
    pub source_path: String,
    /// Worker count.
    pub worker_count: Option<u32>,
}

/// Domain blocker specific to Evaluation commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvaluationBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    SourceRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Evaluation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationResult {
    /// Forced roots count.
    pub forced_roots_count: u32,
    /// Exported bytes count.
    pub emitted_bytes_count: u64,
    /// Streamed frames count.
    pub streamed_frames_count: u32,
}

/// Terminal Evaluation outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationOutcome {
    /// The operation ran and produced a result.
    Completed(EvaluationResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<EvaluationBlocker>),
}

/// Port: execute one typed Evaluation command.
pub trait EvaluationPort {
    fn run(&mut self, request: &EvaluationCommand) -> Result<EvaluationOutcome, CapabilityError>;
}

/// Validate one Evaluation command before any port is called.
pub fn validate_evaluation(request: &EvaluationCommand) -> Vec<EvaluationBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_EVALUATION_BLOCKERS);
    let mut blockers: Vec<EvaluationBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(EvaluationBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "evaluation",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(EvaluationBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(EvaluationBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_EVALUATION_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(EvaluationBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, EvaluationOperation::Evaluate | EvaluationOperation::Force)
        && request.source_path.trim().is_empty()
    {
        blockers.push(EvaluationBlocker::SourceRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_EVALUATION_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> EvaluationCommand {
        EvaluationCommand {
            root: String::from("evaluation"),
            operation: EvaluationOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            source_path: String::from("src/main.ncl"),
            worker_count: Some(1),
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_evaluation(&request).is_empty());
        assert_eq!(EvaluationOperation::all().len(), 4);
        let declared_required =
            EvaluationOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 1);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_evaluation(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, EvaluationBlocker::Domain(_))));
        assert!(blockers.contains(&EvaluationBlocker::MissingSubject));
        assert!(!EvaluationOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.source_path = String::new();
        request.operation = EvaluationOperation::Evaluate;
        let blockers = validate_evaluation(&request);
        assert!(blockers.contains(&EvaluationBlocker::SourceRequired));
        assert!(!blockers.is_empty());
        let empty_labels = EvaluationOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
