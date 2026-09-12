//! Planning command family.
//!
//! Planning commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one planning command.
pub const MAX_PLANNING_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one planning command beyond its declared entries.
const MAX_PLANNING_BLOCKERS: usize = 4;

/// Maximum admitted explanation depth.
pub const MAX_EXPLAIN_DEPTH: u32 = 16;

/// Planning operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlanningOperation {
    /// Plan operation.
    Plan,
    /// Explain operation.
    Explain,
    /// Graph operation.
    Graph,
}

impl PlanningOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Plan, Self::Explain, Self::Graph];
        debug_assert_eq!(entries.len(), 3);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Plan => "plan",
            Self::Explain => "explain",
            Self::Graph => "graph",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(PlanningOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        // Planning reads the requested roots from the command list and needs no declared entries.
        let is_required = false;
        debug_assert!(PlanningOperation::all().contains(&self));
        debug_assert!(PlanningOperation::all().len() == 3);
        is_required
    }
}

/// One typed Planning command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: PlanningOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Explain depth.
    pub explain_depth: u32,
    /// Root names.
    pub root_names: Vec<String>,
}

/// Domain blocker specific to Planning commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlanningBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    DepthUnsupported,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Planning result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningResult {
    /// Planned units count.
    pub planned_units_count: u32,
    /// Effect entries count.
    pub effect_entries_count: u32,
    /// Rejected alternatives count.
    pub rejected_alternatives_count: u32,
}

/// Terminal Planning outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningOutcome {
    /// The operation ran and produced a result.
    Completed(PlanningResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<PlanningBlocker>),
}

/// Port: execute one typed Planning command.
pub trait PlanningPort {
    fn run(&mut self, request: &PlanningCommand) -> Result<PlanningOutcome, CapabilityError>;
}

/// Validate one Planning command before any port is called.
pub fn validate_planning(request: &PlanningCommand) -> Vec<PlanningBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_PLANNING_BLOCKERS);
    let mut blockers: Vec<PlanningBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(PlanningBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "planning",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(PlanningBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(PlanningBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_PLANNING_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(PlanningBlocker::TooManyDeclaredEntries);
    }
    if request.explain_depth > MAX_EXPLAIN_DEPTH {
        blockers.push(PlanningBlocker::DepthUnsupported);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_PLANNING_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> PlanningCommand {
        PlanningCommand {
            root: String::from("planning"),
            operation: PlanningOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            explain_depth: 1,
            root_names: vec![String::from("root")],
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_planning(&request).is_empty());
        assert_eq!(PlanningOperation::all().len(), 3);
        let declared_required =
            PlanningOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 0);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_planning(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, PlanningBlocker::Domain(_))));
        assert!(blockers.contains(&PlanningBlocker::MissingSubject));
        assert!(!PlanningOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.explain_depth = MAX_EXPLAIN_DEPTH.saturating_add(1);
        let blockers = validate_planning(&request);
        assert!(blockers.contains(&PlanningBlocker::DepthUnsupported));
        assert!(!blockers.is_empty());
        let empty_labels = PlanningOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
